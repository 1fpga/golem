//! Types for Cyclone V devices.
//! Derived from https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/index_frames.html
//!
//! These types are used to access the FPGA Manager registers and other Cyclone V specific
//! registers. The code should be platform-agnostic to allow for tests. This means no
//! assembly or architecture specific libraries.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

use core::fmt;
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
use memory::MemoryMapper;

pub mod ctrl;
pub mod gpio_ext_porta;
pub mod memory;
pub mod stat;

/// Absolute addresses for the SoC FPGA.
mod addresses {
    pub const SOCFPGA_BASE_ADDRESS: usize = 0xFF000000;
    pub const SOCFPGA_FPGAMGRREGS_ADDRESS: usize = 0xFF706000;
    pub const SOCFPGA_FPGAMGRDATA_ADDRESS: usize = 0xFFB90000;
}

/// A list of offsets from the base address.
mod offsets {
    use super::addresses::*;

    pub const SOCFPGA_FPGAMGRREGS_OFFSET: usize =
        SOCFPGA_FPGAMGRREGS_ADDRESS - SOCFPGA_BASE_ADDRESS;
    pub const SOCFPGA_FPGAMGRDATA_OFFSET: usize =
        SOCFPGA_FPGAMGRDATA_ADDRESS - SOCFPGA_BASE_ADDRESS;
}

const FPGA_TIMEOUT_CNT: usize = 0x1000000;

#[derive(Debug)]
pub struct SocFpga<M: MemoryMapper> {
    pub memory: M,
}

extern "C" {
    static mut map_base: *mut u8;
}

#[cfg(feature = "std")]
impl Default for SocFpga<memory::DevMemMemoryMapper> {
    fn default() -> Self {
        // Uses `/dev/mem`.
        // TODO: move this to a separate trait and library so people not using Linux can still use
        // this library.

        let mut memory =
            memory::DevMemMemoryMapper::create(addresses::SOCFPGA_BASE_ADDRESS, 0x0100_0000)
                .expect("Could not create memory mapper");

        // TODO: remove this when the fpga code from CPP is gone.
        unsafe { map_base = memory.as_mut_ptr() };

        Self::new(memory)
    }
}

impl<M: MemoryMapper> SocFpga<M> {
    pub fn new(memory: M) -> Self {
        Self { memory }
    }

    pub fn fpga_manager_mut(&mut self) -> &mut FpgaManagerRegs {
        unsafe {
            &mut *((self
                .memory
                .as_mut_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET))
                as *mut FpgaManagerRegs)
        }
    }
    pub fn fpga_manager(&self) -> &FpgaManagerRegs {
        unsafe {
            &*((self
                .memory
                .as_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET)) as *const FpgaManagerRegs)
        }
    }
}

#[derive(Copy, Clone)]
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

#[cfg(feature = "std")]
impl fmt::Debug for FpgaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_tuple("FpgaError");

        fmt.field(&(*self as u32));
        let err: &'static str = self.into();
        fmt.field(&err);

        fmt.finish()
    }
}

impl fmt::Display for FpgaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err: &'static str = self.into();
        write!(f, "{}", err)
    }
}

#[cfg(feature = "std")]
impl From<FpgaError> for String {
    fn from(value: FpgaError) -> Self {
        let err: &'static str = (&value).into();
        err.to_string()
    }
}

/// The FPGA Manager is a hardware block that manages the FPGA configuration
/// process. It is responsible for configuring the FPGA with the image stored
/// in the flash memory. It also provides a mechanism to reconfigure the FPGA
/// with a new image stored in the flash memory.
///
/// This structure must be aligned exactly with the hardware registers.
/// Accessor methods should be used to access the registers with volatile
/// calls to prevent the compiler from optimizing away the reads and writes.
///
/// See https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/sfo1410067808053.html#sfo1410067808053
#[repr(C)]
pub struct FpgaManagerRegs {
    // FPGA Manager Module
    /// Status Register
    stat: u32, // 0x00
    /// Control Register
    ctrl: u32,
    /// Data Clock Count Register
    dclkcnt: u32,
    /// Data Clock Status Register
    dclkstat: u32,
    /// General Purpose Output Register
    gpo: u32,
    /// General Purpose Input Register (RO)
    gpi: u32,
    /// Miscellaneous Input Register (RO)
    misci: u32,

    // Padding
    _pad_0x1c_0x82c: [u32; 517],

    // Configuration Monitor (MON) Registers
    /// GPIO Interrupt Enable Register
    gpio_inten: u32,
    /// GPIO Interrupt Mask Register
    gpio_intmask: u32,
    /// GPIO Interrupt Type Level Register
    gpio_inttype_level: u32,
    /// GPIO Interrupt Polarity Register
    gpio_int_polarity: u32,
    /// GPIO Interrupt Status Register
    gpio_intstatus: u32,
    /// GPIO Raw Interrupt Status Register
    gpio_raw_intstatus: u32,

    // Padding
    _pad_0x848: u32,

    /// GPIO Port A Clear Interrupt Register
    gpio_porta_eoi: u32,
    /// GPIO Port A External Port Register
    gpio_ext_porta: u32,

    // Padding
    _pad_0x854_0x85c: [u32; 3],

    /// GPIO Level Synchronization Register
    gpio_ls_sync: u32,

    // Padding
    _pad_0x864_0x868: [u32; 2],

    /// GPIO Version ID Code Register
    gpio_ver_id_code: u32,
    /// GPIO Configuration Register 2
    gpio_config_reg2: u32,
    /// GPIO Configuration Register 1
    gpio_config_reg1: u32,
}

impl FpgaManagerRegs {
    #[inline]
    fn wait_for(&mut self, mut f: impl FnMut(&mut Self) -> bool) -> Result<(), ()> {
        for _ in 0..FPGA_TIMEOUT_CNT {
            if f(self) {
                return Ok(());
            }
        }
        Err(())
    }

    fn as_mut_ptr(&mut self) -> *mut Self {
        self as *mut Self
    }

    /// Set the data clock count register. Returns Ok if the register
    /// was properly set, and Err if a timeout occured checking for
    /// the status.
    #[inline]
    pub fn set_dclkcnt(&mut self, value: u32) -> Result<(), FpgaError> {
        unsafe {
            // Clear any existing done status.
            if read_volatile(addr_of!(self.dclkstat)) != 0 {
                write_volatile(addr_of_mut!(self.dclkstat), 0x1);
            }

            // Write the dclkcnt register.
            write_volatile(addr_of_mut!(self.dclkcnt), value);
        }

        // Wait for the done status to be set.
        self.wait_for(|mgr| unsafe {
            if read_volatile(addr_of!(mgr.dclkstat)) == 0 {
                return true;
            }

            write_volatile(addr_of_mut!(mgr.dclkstat), 0x1);
            false
        })
        .map_err(|_| FpgaError::Timeout)
    }

    #[inline]
    pub fn set_gpo(&mut self, value: u32) {
        unsafe {
            write_volatile(addr_of_mut!(self.gpo), value);
        }
    }

    /// Get the current set GPO register.
    #[inline]
    pub fn gpo(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.gpo)) }
    }

    /// Get the GPI register.
    #[inline]
    pub fn gpi(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.gpi)) }
    }

    #[inline]
    pub fn gpio_porta_eoi(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.gpio_porta_eoi)) }
    }

    #[inline]
    pub fn set_gpio_porta_eoi(&mut self, value: u32) {
        unsafe {
            write_volatile(addr_of_mut!(self.gpio_porta_eoi), value);
        }
    }

    #[inline]
    pub fn gpio_ext_porta(&self) -> gpio_ext_porta::GpioExtPorta {
        gpio_ext_porta::GpioExtPorta(unsafe { read_volatile(addr_of!(self.gpio_ext_porta)) })
    }

    #[inline]
    pub fn status(&self) -> stat::StatusRegister {
        stat::StatusRegister(unsafe { read_volatile(addr_of!(self.stat)) })
    }

    #[inline]
    pub fn set_status(&mut self, value: stat::StatusRegister) {
        unsafe { write_volatile(addr_of_mut!(self.stat), value.0) }
    }

    #[inline]
    pub fn update_status(&mut self, f: impl FnOnce(&mut stat::StatusRegister)) {
        let mut value = self.status();
        f(&mut value);
        self.set_status(value);
    }

    #[inline]
    pub fn ctrl(&self) -> ctrl::FpgaConfigurationControl {
        ctrl::FpgaConfigurationControl(unsafe { read_volatile(addr_of!(self.ctrl)) })
    }

    #[inline]
    pub fn set_ctrl(&mut self, value: ctrl::FpgaConfigurationControl) {
        unsafe {
            write_volatile(addr_of_mut!(self.ctrl), value.0);
        }
    }

    #[inline]
    pub fn update_ctrl(&mut self, f: impl FnOnce(&mut ctrl::FpgaConfigurationControl)) {
        let mut value = self.ctrl();
        f(&mut value);
        self.set_ctrl(value);
    }
}

impl FpgaManagerRegs {
    pub fn load_rbf(&mut self, program: &[u8]) -> Result<(), FpgaError> {
        self.init_program()?;
        self.write_program(program)?;
        self.poll_configuration_done()?;

        let init_result = self.poll_init_phase();
        match init_result {
            Ok(()) => {}
            Err(e) => {
                return Err(e);
            }
        }

        self.poll_user_mode()?;

        Ok(())
    }

    #[inline]
    fn init_program(&mut self) -> Result<(), FpgaError> {
        let msel = self.status().msel();
        eprintln!("msel: {:?}", msel);

        // Set the cfg width.
        eprintln!("ctrl1: {:?}", self.ctrl());
        self.update_ctrl(|ctrl| {
            eprintln!("ctrl2: {:?}", ctrl);
            if msel.is_32_bits() {
                ctrl.set_cfgwdth(ctrl::FpgaCtrlCfgWidth::Passive32Bit);
            } else {
                ctrl.set_cfgwdth(ctrl::FpgaCtrlCfgWidth::Passive16Bit);
            }

            ctrl.set_cdratio(msel.cd_ratio());

            // To enable FPGA Manager configuration.
            ctrl.set_nce(ctrl::FpgaCtrlNce::Enabled);

            // To enable FPGA Manager drive over configuration line.
            ctrl.set_en(ctrl::FpgaCtrlEn::FpgaManager);

            // Put FPGA into reset phase.
            ctrl.set_nconfigpull(true);
        });
        eprintln!("ctrl3: {:?}", self.ctrl());

        // (1) wait until FPGA enter reset phase
        self.wait_for(|mgr| mgr.status().mode() == stat::StatusRegisterMode::ResetPhase)
            .map_err(|_| FpgaError::CouldNotReset)?;

        // Release FPGA from reset phase.
        self.update_ctrl(|ctrl| ctrl.set_nconfigpull(false));

        // (2) wait until FPGA enter configuration phase
        self.wait_for(|mgr| mgr.status().mode() == stat::StatusRegisterMode::ConfigPhase)
            .map_err(|_| FpgaError::CouldNotEnterConfigurationPhase)?;

        // Clear all interrupts in CB Monitor.
        self.set_gpio_porta_eoi(0xFFF);

        // Enable AXI configuration
        self.update_ctrl(|ctrl| ctrl.set_axicfgen(true));

        Ok(())
    }

    /// Write the RBF program to the FPGA.
    /// This code is copied from u-boot from [here](
    /// https://github.com/u-boot/u-boot/blob/master/drivers/fpga/socfpga.c#L44), converted to
    /// Rust. The original code is licensed under the GPL-2.0+.
    ///
    /// In our case, we also changed the registers used for the load/store as LLVM reserves
    /// the r6 register for internal use (stack frames, etc). We could not compile it while
    /// using r6.
    ///
    /// This requires the ARM architecture to be enabled.
    #[inline(never)]
    pub fn write_program(&mut self, program: &[u8]) -> Result<(), FpgaError> {
        if program.is_empty() {
            return Ok(());
        }

        let dst = unsafe {
            // We have to offset REGS since we are not totally aligned to the start of the
            // memory. So the DATA memory is actually at SELF - REGS + DATA.
            (self.as_mut_ptr() as *mut u8)
                .add(offsets::SOCFPGA_FPGAMGRDATA_OFFSET - offsets::SOCFPGA_FPGAMGRREGS_OFFSET)
        } as *mut u32;

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
                in(reg) dst,
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
                write_volatile(dst as *mut u8, *v);
            }
            for v in shorts {
                write_volatile(dst, *v);
            }
            for v in suffix {
                write_volatile(dst as *mut u8, *v);
            }
        }

        Ok(())
    }

    #[inline]
    fn poll_configuration_done(&mut self) -> Result<(), FpgaError> {
        // // (3) wait until full config done.
        // self.wait(|mgr| mgr.gpio_ext_porta().ns() || mgr.gpio_ext_porta().cd())
        //     .map_err(|_| FpgaError::CouldNotConfigure)?;
        //
        // // Disable AXI configuration
        // self.update_ctrl(|ctrl| ctrl.set_axicfgen(false));

        extern "C" {
            fn fpgamgr_program_poll_cd() -> core::ffi::c_int;
        }
        unsafe {
            assert_eq!(fpgamgr_program_poll_cd(), 0);
        }

        Ok(())
    }

    #[inline]
    fn poll_init_phase(&mut self) -> Result<(), FpgaError> {
        // Additional clocks for the CB to enter initialization phase
        self.set_dclkcnt(0x4)?;

        // (4) wait until FPGA enter init phase
        self.wait_for(|mgr| {
            matches!(
                mgr.status().mode(),
                stat::StatusRegisterMode::InitPhase | stat::StatusRegisterMode::UserMode
            )
        })
        .map_err(|_| FpgaError::CouldNotEnterInitPhase)?;

        Ok(())
    }

    #[inline]
    fn poll_user_mode(&mut self) -> Result<(), FpgaError> {
        self.set_dclkcnt(0x5000)?;

        // (5) wait until FPGA enter user mode
        self.wait_for(|mgr| mgr.status().mode() == stat::StatusRegisterMode::UserMode)
            .map_err(|_| FpgaError::CouldNotEnterUserMode)?;

        // To release FPGA Manager drive over configuration line.
        self.update_ctrl(|ctrl| ctrl.set_en(ctrl::FpgaCtrlEn::FpgaConfigurationPins));

        Ok(())
    }
}

#[test]
fn update_ctrl() {
    let mut region = vec![0; 0x1000000];
    let mapper = memory::RegionMemoryMapper::new(&mut region);
    let mut soc = SocFpga::new(mapper);

    let mgr = soc.fpga_manager_mut();

    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaConfigurationPins);
    mgr.update_ctrl(|ctrl| ctrl.set_en(ctrl::FpgaCtrlEn::FpgaManager));
    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaManager);
}
