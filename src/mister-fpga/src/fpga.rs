use std::cell::UnsafeCell;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use strum::{Display, EnumIter, FromRepr};
use tracing::{debug, error, info, trace};

use cyclone_v::fpgamgrregs::ctrl::{FpgaCtrlCfgWidth, FpgaCtrlEn, FpgaCtrlNce};
use cyclone_v::fpgamgrregs::stat::StatusRegisterMode;
use cyclone_v::memory::DevMemMemoryMapper;
pub use program::Program;
pub use spi::*;

use crate::fpga::osd_io::{OsdDisable, OsdEnable};

mod program;
mod spi;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum FpgaError {
    Timeout = 1,
    CouldNotReset,
    CouldNotEnterConfigurationPhase,
    CouldNotConfigure,
    CouldNotEnterInitPhase,
    CouldNotEnterUserMode,
    IoError,
}

impl From<FpgaError> for &'static str {
    fn from(value: FpgaError) -> Self {
        (&value).into()
    }
}

impl From<&FpgaError> for &'static str {
    fn from(value: &FpgaError) -> Self {
        match value {
            FpgaError::Timeout => "FPGA Timeout",
            FpgaError::CouldNotReset => "Could not reset FPGA",
            FpgaError::CouldNotEnterConfigurationPhase => "Could not enter configuration phase",
            FpgaError::CouldNotConfigure => "Could not configure FPGA",
            FpgaError::CouldNotEnterInitPhase => "Could not enter init phase",
            FpgaError::CouldNotEnterUserMode => "Could not enter user mode",
            FpgaError::IoError => "I/O Error",
        }
    }
}

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

// TODO: Remove this when we're done re-writing fpga_io.cpp
// This is needed for the FFI implementations to work.
// Also, remove the Rc<RefCell<_>> below.
static mut FPGA_SINGLETON: Option<MisterFpga> = None;

/// FPGA core type.
#[derive(Display, Debug, Eq, PartialEq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum CoreType {
    /// Core type value should be unlikely to be returned by broken cores.
    CoreTypeUnknown = 0x55,

    /// Generic Core. Called CORE_TYPE_8BIT in Main_MiSTer.
    CoreTypeGeneric = 0xA4,

    /// Sharp MZ Series.
    CoreTypeSharpMz = 0xA7,

    /// Generic Core using dual SDRAM. Called CORE_TYPE_8BIT2 in Main_MiSTer.
    CoreTypeGenericDualSdram = 0xA8,
}

impl CoreType {
    pub fn is_dual_sdram(&self) -> bool {
        *self == CoreType::CoreTypeGenericDualSdram
    }

    pub fn is_generic(&self) -> bool {
        *self == CoreType::CoreTypeGeneric || *self == CoreType::CoreTypeGenericDualSdram
    }
}

/// The interface type of the core.
#[derive(Display, Debug, Eq, PartialEq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum CoreInterfaceType {
    /// 8-bit SPI bus.
    SpiBus8Bit = 0,

    /// 16-bit SPI bus.
    SpiBus16Bit = 1,
}

impl CoreInterfaceType {
    pub fn is_wide(&self) -> bool {
        *self == CoreInterfaceType::SpiBus16Bit
    }
}

#[derive(Debug, Clone)]
pub struct MisterFpga {
    soc: Arc<UnsafeCell<cyclone_v::SocFpga<DevMemMemoryMapper>>>,
    spi: Spi<DevMemMemoryMapper>,
}

// SAFETY:
// Since the FPGA is using memory-mapped I/O, it is not safe to send it to another thread.
unsafe impl Send for MisterFpga {}
unsafe impl Sync for MisterFpga {}

// OSD specific functions.
impl MisterFpga {
    pub fn osd_enable(&mut self) {
        let _ = self.spi_mut().execute(OsdEnable);
    }
    pub fn osd_disable(&mut self) {
        let _ = self.spi_mut().execute(OsdDisable);
    }
}

impl MisterFpga {
    fn new(soc: Arc<UnsafeCell<cyclone_v::SocFpga<DevMemMemoryMapper>>>) -> Self {
        Self {
            soc: soc.clone(),
            spi: Spi::new(soc),
        }
    }

    #[inline]
    #[allow(clippy::mut_from_ref)]
    fn soc_mut(&self) -> &mut cyclone_v::SocFpga<DevMemMemoryMapper> {
        unsafe { &mut (*self.soc.get()) }
    }

    fn regs(&self) -> &cyclone_v::fpgamgrregs::FpgaManagerRegs {
        self.soc_mut().regs()
    }

    fn regs_mut(&mut self) -> &mut cyclone_v::fpgamgrregs::FpgaManagerRegs {
        self.soc_mut().regs_mut()
    }

    pub fn init() -> Result<Self, &'static str> {
        unsafe {
            if INITIALIZED.load(Ordering::Relaxed) {
                const MSG: &str = "FPGA already initialized. This is an error.";
                error!("{}", MSG);
                return Err(MSG);
            }

            info!("Initializing FPGA");

            let soc = cyclone_v::SocFpga::default();

            // TODO: Remove UnsafeCell here.
            #[allow(clippy::arc_with_non_send_sync)]
            let soc = Arc::new(UnsafeCell::new(soc));
            let mut fpga = Self::new(soc.clone());

            fpga.regs_mut().set_gpo(0);

            FPGA_SINGLETON = Some(fpga.clone());

            INITIALIZED.store(true, Ordering::Relaxed);
            Ok(fpga)
        }
    }

    pub fn spi(&self) -> &Spi<DevMemMemoryMapper> {
        &self.spi
    }

    pub fn spi_mut(&mut self) -> &mut Spi<DevMemMemoryMapper> {
        &mut self.spi
    }

    pub fn core_type(&mut self) -> Option<CoreType> {
        let regs = self.regs_mut();

        let gpo = regs.gpo() & 0x7FFF_FFFF;
        regs.set_gpo(0);
        let core_type: u32 = regs.gpi();
        regs.set_gpo(gpo | 0x80000000);

        if (core_type & 0xFFFFFF00) != 0x5CA62300 {
            error!("FPGA core type mismatch");
            None
        } else {
            CoreType::from_repr((core_type & 0xFF) as u8)
        }
    }

    pub fn core_interface_type(&self) -> Option<CoreInterfaceType> {
        let manager = self.regs();

        CoreInterfaceType::from_repr((manager.gpi() >> 16 & 1) as u8)
    }

    pub fn core_io_version(&self) -> Option<u8> {
        let manager = self.regs();

        let version = manager.gpi() >> 18 & 0b00000011;
        if version == 0 {
            error!("FPGA core IO version mismatch");
            None
        } else {
            Some(version as u8)
        }
    }

    /// Check whether the FPGA is ready to be accessed.
    #[inline]
    pub fn is_ready(&self) -> bool {
        // Check twice to avoid false/timing glitches.
        for _ in 0..2 {
            if !self.regs().gpio_ext_porta().id() {
                return false;
            }
        }

        return self.regs().stat().mode() == StatusRegisterMode::UserMode;
    }

    #[inline]
    fn is_ready_quick(&self) -> bool {
        (self.regs().gpi() as i32) >= 0
    }

    /// Send a reset signal to the core.
    pub fn core_reset(&mut self) {
        let fpga_manager = self.regs_mut();

        // Core Reset.
        let gpo = fpga_manager.gpo() & (!0xC000_0000);
        fpga_manager.set_gpo(gpo | 0x4000_0000);
    }

    #[inline]
    pub(super) fn wait_to_reset(&mut self) {
        let manager = self.regs_mut();

        debug!("FPGA is not ready. JTAG uploading?");
        info!("Waiting for FPGA to be ready...");

        // Send the reset signal to the FPGA.
        let gpo = manager.gpo() & (!0xC0000000);
        manager.set_gpo(gpo | 0x40000000);

        while !self.is_ready() {
            std::thread::sleep(Duration::from_millis(10));
        }

        // TODO: is this needed here?
        // unsafe {
        //     reboot(0);
        // }
    }

    /// Wait for the FPGA to be ready. This requires a mutable reference if the
    /// FPGA is not ready.
    pub fn wait_for_ready(&mut self) {
        while !self.is_ready_quick() {
            self.wait_to_reset();
        }
    }

    #[inline]
    fn wait_for(&mut self, mut f: impl FnMut(&mut Self) -> bool) -> Option<()> {
        let now = Instant::now();
        let timeout = Duration::from_secs(10);

        while now.elapsed() < timeout {
            if f(self) {
                return Some(());
            }

            std::thread::sleep(Duration::from_millis(1));
        }

        None
    }

    #[inline]
    pub fn enable_bridge(&mut self) {
        self.soc_mut()
            .sdr_mut()
            .update_fpgaportrst(|portrst| portrst.set_portrstn(0x3FFF));
        self.soc_mut().rstmgr_mut().set_brgmodrst(0);

        let mut remap = cyclone_v::l3regs::remap::Remap(0);
        remap.set_mpuzero(true);
        remap.set_hps2fpga(true);
        remap.set_lwhps2fpga(true);
        self.soc_mut().l3regs_mut().set_remap(remap);
    }

    #[inline]
    pub fn disable_bridge(&mut self) {
        let soc = self.soc_mut();
        soc.sysmgr_mut().set_fpgaintfgrp_module(0);
        soc.sdr_mut()
            .update_fpgaportrst(|portrst| portrst.set_portrstn(0));
        soc.rstmgr_mut().set_brgmodrst(7);
        let mut remap = cyclone_v::l3regs::remap::Remap(0);
        remap.set_mpuzero(true);
        soc.l3regs_mut().set_remap(remap);
    }

    /// Set the data clock count register. Returns Ok if the register
    /// was properly set, and Err if a timeout occurred checking for
    /// the status.
    #[inline]
    pub fn set_dclkcnt(&mut self, value: u32) -> Result<(), FpgaError> {
        // Clear any existing done status.
        if self.regs_mut().dclkstat() != 0 {
            self.regs_mut().set_dclkstat(0x1);
        }

        // Write the dclkcnt register.
        self.regs_mut().set_dclkcnt(value);

        // Wait for the done status to be set.
        self.wait_for(|fpga| {
            if fpga.regs_mut().dclkstat() == 0 {
                true
            } else {
                fpga.regs_mut().set_dclkstat(0x1);
                false
            }
        })
        .ok_or(FpgaError::Timeout)
    }

    pub fn load(&mut self, program: impl Program) -> Result<(), FpgaError> {
        program.load(self)
    }

    pub(crate) fn load_rbf_bytes(&mut self, bytes: &[u8]) -> Result<(), FpgaError> {
        let start = Instant::now();
        self.disable_bridge();

        debug!("Initializing FPGA...");
        self.init_program()?;
        debug!("Writing program...");
        let now = Instant::now();
        self.write_program(bytes)?;
        trace!("Program written in {}ms", now.elapsed().as_millis());

        debug!("Configuration and initialization.");
        self.enable_bridge();

        debug!("Polling configuration done...");
        self.poll_configuration_done()?;
        debug!("Polling init phase...");
        self.poll_init_phase()?;
        debug!("Polling user mode...");
        self.poll_user_mode()?;

        trace!(
            "Done loading program. In {}ms.",
            start.elapsed().as_millis()
        );

        Ok(())
    }

    #[inline]
    fn init_program(&mut self) -> Result<(), FpgaError> {
        let msel = self.regs_mut().stat().msel();

        // Set the cfg width.
        self.regs_mut().update_ctrl(|ctrl| {
            if msel.is_32_bits() {
                ctrl.set_cfgwdth(FpgaCtrlCfgWidth::Passive32Bit);
            } else {
                ctrl.set_cfgwdth(FpgaCtrlCfgWidth::Passive16Bit);
            }

            ctrl.set_cdratio(msel.cd_ratio());

            // To enable FPGA Manager configuration.
            ctrl.set_nce(FpgaCtrlNce::Enabled);

            // To enable FPGA Manager drive over configuration line.
            ctrl.set_en(FpgaCtrlEn::FpgaManager);

            // Put FPGA into reset phase.
            ctrl.set_nconfigpull(true);
        });

        // (1) wait until FPGA enter reset phase
        self.wait_for(|fpga| fpga.regs_mut().stat().mode() == StatusRegisterMode::ResetPhase)
            .ok_or(FpgaError::CouldNotReset)?;

        // Release FPGA from reset phase.
        self.regs_mut()
            .update_ctrl(|ctrl| ctrl.set_nconfigpull(false));

        // (2) wait until FPGA enter configuration phase
        self.wait_for(|fpga| fpga.regs_mut().stat().mode() == StatusRegisterMode::ConfigPhase)
            .ok_or(FpgaError::CouldNotEnterConfigurationPhase)?;

        // Clear all interrupts in CB Monitor.
        self.regs_mut().set_gpio_porta_eoi(0xFFF);

        // Enable AXI configuration
        self.regs_mut().update_ctrl(|ctrl| ctrl.set_axicfgen(true));

        Ok(())
    }

    /// Write the RBF program to the FPGA.
    #[inline(never)]
    pub fn write_program(&mut self, program: impl Read) -> Result<(), FpgaError> {
        let program = program
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_err| FpgaError::IoError)?;
        if program.is_empty() {
            return Ok(());
        }

        let data = unsafe { self.soc_mut().data_ptr_mut() } as *mut u32;
        unsafe {
            let (prefix, shorts, suffix) = program.align_to::<u32>();

            for v in prefix {
                std::ptr::write_volatile(data as *mut u8, *v);
            }
            for v in shorts {
                std::ptr::write_volatile(data, *v);
            }
            for v in suffix {
                std::ptr::write_volatile(data as *mut u8, *v);
            }
        }

        Ok(())
    }

    #[inline]
    fn poll_configuration_done(&mut self) -> Result<(), FpgaError> {
        // (3) wait until full config done.
        self.wait_for(|fpga| {
            fpga.regs_mut().gpio_ext_porta().ns() || fpga.regs_mut().gpio_ext_porta().cd()
        })
        .ok_or(FpgaError::CouldNotConfigure)?;

        // Disable AXI configuration
        self.regs_mut().update_ctrl(|ctrl| ctrl.set_axicfgen(false));

        Ok(())
    }

    #[inline]
    fn poll_init_phase(&mut self) -> Result<(), FpgaError> {
        // Additional clocks for the CB to enter initialization phase
        self.set_dclkcnt(0x4)?;

        // (4) wait until FPGA enter init phase
        self.wait_for(|fpga| {
            matches!(
                fpga.regs_mut().stat().mode(),
                StatusRegisterMode::InitPhase | StatusRegisterMode::UserMode
            )
        })
        .ok_or(FpgaError::CouldNotEnterInitPhase)?;

        Ok(())
    }

    #[inline]
    fn poll_user_mode(&mut self) -> Result<(), FpgaError> {
        self.set_dclkcnt(0x5000)?;

        // (5) wait until FPGA enter user mode
        self.wait_for(|fpga| fpga.regs_mut().stat().mode() == StatusRegisterMode::UserMode)
            .ok_or(FpgaError::CouldNotEnterUserMode)?;

        // To release FPGA Manager drive over configuration line.
        self.regs_mut()
            .update_ctrl(|ctrl| ctrl.set_en(FpgaCtrlEn::FpgaConfigurationPins));

        Ok(())
    }
}
