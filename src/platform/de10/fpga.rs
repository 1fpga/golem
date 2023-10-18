use crate::platform::de10::osd::{OSD_CMD_DISABLE, OSD_CMD_ENABLE};
use crate::platform::de10::spi as ffi_spi;
use cyclone_v::fpgamgrregs::ctrl::{FpgaCtrlCfgWidth, FpgaCtrlEn, FpgaCtrlNce};
use cyclone_v::fpgamgrregs::stat::StatusRegisterMode;
use cyclone_v::memory::DevMemMemoryMapper;
use std::cell::UnsafeCell;
use std::ffi::c_int;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use strum::{Display, EnumIter, FromRepr};
use tracing::{debug, error, info, trace};

mod framebuffer;
mod spi;

pub use spi::*;

extern "C" {
    pub fn reboot(cold: c_int);
}

/// Functions made available to the C code.
/// TODO: remove these when the fpga code from CPP is gone.
pub mod ffi {
    use super::FPGA_SINGLETON;
    use libc::{c_int, c_ulong};
    use tracing::error;

    #[no_mangle]
    extern "C" fn fpga_core_id() -> c_int {
        unsafe {
            FPGA_SINGLETON
                .as_mut()
                .unwrap()
                .core_type()
                .map(|v| v as c_int)
                .unwrap_or_else(|| {
                    error!("FPGA core type mismatch");
                    -1
                })
        }
    }

    #[no_mangle]
    extern "C" fn fpga_get_fio_size() -> c_int {
        unsafe {
            match FPGA_SINGLETON.as_mut().unwrap().core_interface_type() {
                Some(super::CoreInterfaceType::SpiBus8Bit) => 0,
                Some(super::CoreInterfaceType::SpiBus16Bit) => 1,
                _ => -1,
            }
        }
    }

    #[no_mangle]
    extern "C" fn fpga_get_io_version() -> c_int {
        unsafe {
            FPGA_SINGLETON
                .as_mut()
                .unwrap()
                .core_io_version()
                .map(|v| v as c_int)
                .unwrap_or(-1)
        }
    }

    #[no_mangle]
    extern "C" fn fpga_wait_to_reset() {
        unsafe {
            FPGA_SINGLETON.as_mut().unwrap().wait_to_reset();
        }
    }

    #[no_mangle]
    unsafe extern "C" fn fpgamgr_dclkcnt_set_rust(count: c_ulong) -> c_int {
        FPGA_SINGLETON
            .as_mut()
            .unwrap()
            .set_dclkcnt(count as u32)
            .map_or(
                -3447, // aka -ETIMEOUT
                |_| 0,
            )
    }

    #[no_mangle]
    unsafe extern "C" fn fpgamgr_program_write_rust(rbf_data: *const u8, rbf_size: c_ulong) {
        let program = std::slice::from_raw_parts(rbf_data, rbf_size as usize);
        FPGA_SINGLETON
            .as_mut()
            .unwrap()
            .write_program(program)
            .unwrap();
    }

    #[no_mangle]
    unsafe extern "C" fn fpga_spi(word: u16) -> u16 {
        FPGA_SINGLETON.as_mut().unwrap().spi_mut().write(word)
    }

    // #[no_mangle]
    // unsafe extern "C" fn spi_w(word: u16) -> u16 {
    //     FPGA_SINGLETON.as_mut().unwrap().spi_mut().write(word)
    // }

    #[no_mangle]
    unsafe extern "C" fn fpga_spi_en(mask: u32, en: u32) {
        if en != 0 {
            FPGA_SINGLETON.as_mut().unwrap().spi_mut().enable_u32(mask);
        } else {
            FPGA_SINGLETON.as_mut().unwrap().spi_mut().disable_u32(mask);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn DisableIO() {
        FPGA_SINGLETON
            .as_mut()
            .unwrap()
            .spi_mut()
            .disable(super::spi::SpiFeature::IO);
    }

    #[no_mangle]
    unsafe extern "C" fn fpga_spi_fast_block_write(data: *const u16, len: u32) {
        let data = std::slice::from_raw_parts(data, len as usize);
        FPGA_SINGLETON
            .as_mut()
            .unwrap()
            .spi_mut()
            .write_block_16(data);
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum FpgaError {
    Timeout = 1,
    CouldNotReset,
    CouldNotEnterConfigurationPhase,
    CouldNotConfigure,
    CouldNotEnterInitPhase,
    CouldNotEnterUserMode,
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
        }
    }
}

static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

// TODO: Remove this when we're done re-writing fpga_io.cpp
// This is needed for the FFI implementations to work.
// Also, remove the Rc<RefCell<_>> below.
static mut FPGA_SINGLETON: Option<Fpga> = None;

/// FPGA core type.
#[derive(Display, Debug, Eq, PartialEq, EnumIter, FromRepr)]
#[repr(u8)]
pub enum CoreType {
    /// Core type value should be unlikely to be returned by broken cores.
    CoreTypeUnknown = 0x55,

    /// Generic Core.
    CoreTypeGeneric = 0xA4,

    /// Sharp MZ Series.
    CoreTypeSharpMz = 0xA7,

    /// Generic Core using dual SDRAM.
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

extern "C" {
    static mut map_base: *mut u8;
}

#[derive(Debug, Clone)]
pub struct Fpga {
    soc: Rc<UnsafeCell<cyclone_v::SocFpga<DevMemMemoryMapper>>>,
    spi: spi::Spi<DevMemMemoryMapper>,
}

// OSD specific functions.
impl Fpga {
    pub fn osd_enable(&mut self) {
        unsafe {
            // self.spi_mut().write_b(OSD_CMD_ENABLE);
            // self.spi_mut()
            //     .enable(SpiFeature::default().with_osd().with_io().with_fpga());
            ffi_spi::spi_osd_cmd(OSD_CMD_ENABLE);
        }
    }
    pub fn osd_disable(&mut self) {
        unsafe {
            // self.spi_mut()
            //     .disable(SpiFeature::default().with_osd().with_io().with_fpga());
            ffi_spi::spi_osd_cmd(OSD_CMD_DISABLE);
        }
    }
}

impl Fpga {
    fn new(soc: Rc<UnsafeCell<cyclone_v::SocFpga<DevMemMemoryMapper>>>) -> Self {
        Self {
            soc: soc.clone(),
            spi: spi::Spi::new(soc),
        }
    }

    #[inline]
    fn soc_mut(&self) -> &mut cyclone_v::SocFpga<DevMemMemoryMapper> {
        unsafe { &mut (*self.soc.get()) }
    }

    fn regs(&self) -> &cyclone_v::fpgamgrregs::FpgaManagerRegs {
        self.soc_mut().regs()
    }

    fn regs_mut(&mut self) -> &mut cyclone_v::fpgamgrregs::FpgaManagerRegs {
        self.soc_mut().regs_mut()
    }

    pub fn init() -> Result<Self, ()> {
        unsafe {
            if INITIALIZED.load(Ordering::Relaxed) {
                error!("FPGA already initialized. This is an error.");
                return Err(());
            }

            info!("Initializing FPGA");

            let soc = cyclone_v::SocFpga::default();
            let soc = Rc::new(UnsafeCell::new(soc));
            let mut fpga = Self::new(soc.clone());

            // TODO: remove this when the fpga code from CPP is gone.
            map_base = fpga.soc_mut().base_mut();
            fpga.regs_mut().set_gpo(0);

            FPGA_SINGLETON = Some(fpga.clone());

            INITIALIZED.store(true, Ordering::Relaxed);

            Ok(fpga)
        }
    }

    pub fn spi(&self) -> &spi::Spi<DevMemMemoryMapper> {
        &self.spi
    }

    pub fn spi_mut(&mut self) -> &mut spi::Spi<DevMemMemoryMapper> {
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
    pub fn core_reset(&mut self) -> Result<(), ()> {
        let fpga_manager = self.regs_mut();

        // Core Reset.
        let gpo = fpga_manager.gpo() & (!0xC000_0000);
        fpga_manager.set_gpo(gpo | 0x4000_0000);
        Ok(())
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

        unsafe {
            reboot(0);
        }
    }

    /// Wait for the FPGA to be ready. This requires a mutable reference if the
    /// FPGA is not ready.
    pub fn wait_for_ready(&mut self) {
        while !self.is_ready_quick() {
            self.wait_to_reset();
        }
    }

    #[inline]
    fn wait_for(&mut self, mut f: impl FnMut(&mut Self) -> bool) -> Result<(), ()> {
        let now = Instant::now();
        let timeout = Duration::from_secs(3);

        while now.elapsed() < timeout {
            if f(self) {
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(1));
        }

        Err(())
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
    /// was properly set, and Err if a timeout occured checking for
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
        .map_err(|_| FpgaError::Timeout)
    }

    pub fn load_rbf(&mut self, program: &[u8]) -> Result<(), FpgaError> {
        let start = Instant::now();
        self.disable_bridge();

        debug!("Initializing FPGA...");
        self.init_program()?;
        debug!("Writing program...");
        let now = Instant::now();
        self.write_program(program)?;
        trace!("Program written in {}ms", now.elapsed().as_millis());

        debug!("Configuration and initialization.");
        self.enable_bridge();
        self.poll_configuration_done()?;
        self.poll_init_phase()?;
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
            .map_err(|_| FpgaError::CouldNotReset)?;

        // Release FPGA from reset phase.
        self.regs_mut()
            .update_ctrl(|ctrl| ctrl.set_nconfigpull(false));

        // (2) wait until FPGA enter configuration phase
        self.wait_for(|fpga| fpga.regs_mut().stat().mode() == StatusRegisterMode::ConfigPhase)
            .map_err(|_| FpgaError::CouldNotEnterConfigurationPhase)?;

        // Clear all interrupts in CB Monitor.
        self.regs_mut().set_gpio_porta_eoi(0xFFF);

        // Enable AXI configuration
        self.regs_mut().update_ctrl(|ctrl| ctrl.set_axicfgen(true));

        Ok(())
    }

    /// Write the RBF program to the FPGA.
    #[inline(never)]
    pub fn write_program(&mut self, program: &[u8]) -> Result<(), FpgaError> {
        if program.is_empty() {
            return Ok(());
        }

        let data = self.soc_mut().data_mut() as *mut u32;

        // This code is copied from u-boot from [here](
        // https://github.com/u-boot/u-boot/blob/master/drivers/fpga/socfpga.c#L44), converted to
        // Rust. The original code is licensed under the GPL-2.0+.
        //
        // In our case, we also changed the registers used for the load/store as LLVM reserves
        // the r6 register for internal use (stack frames, etc.). We could not compile it while
        // using r6.
        //
        // This requires the ARM architecture to be enabled.
        #[cfg(target_arch = "arm")]
        unsafe {
            let src: u32 = program.as_ptr() as u32;
            let loops32: u32 = program.len() as u32 / 32;

            // Number of loops for 4-byte long copying + trailing bytes.
            let loops4: u32 = {
                let n = (program.len() as u32) % 32;
                let d = 4;
                (n + d - 1) / d
            };

            core::arch::asm!(
            "1: ldmia   {0}!,    {{r0-r5,r7-r8}}",
            "   stmia   {1}!,    {{r0-r5,r7-r8}}",
            "   sub     {1},     #32",
            "   subs    {2}, #1",
            "   bne     1b",
            "   cmp     {3},  #0",
            "   beq     3f",
            "2: ldr     {2}, [{0}],   #4",
            "   str     {2}, [{1}]",
            "   subs    {3},  #1",
            "   bne     2b",
            "3: nop",
            in(reg) src,
            in(reg) data,
            in(reg) loops32,
            in(reg) loops4,
            out("r0") _, out("r1") _, out("r2") _, out("r3") _,
            out("r4") _, out("r5") _, out("r7") _, out("r8") _
            );
        }

        #[cfg(not(target_arch = "arm"))]
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
        .map_err(|_| FpgaError::CouldNotConfigure)?;

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
        .map_err(|_| FpgaError::CouldNotEnterInitPhase)?;

        Ok(())
    }

    #[inline]
    fn poll_user_mode(&mut self) -> Result<(), FpgaError> {
        self.set_dclkcnt(0x5000)?;

        // (5) wait until FPGA enter user mode
        self.wait_for(|fpga| fpga.regs_mut().stat().mode() == StatusRegisterMode::UserMode)
            .map_err(|_| FpgaError::CouldNotEnterUserMode)?;

        // To release FPGA Manager drive over configuration line.
        self.regs_mut()
            .update_ctrl(|ctrl| ctrl.set_en(FpgaCtrlEn::FpgaConfigurationPins));

        Ok(())
    }
}
