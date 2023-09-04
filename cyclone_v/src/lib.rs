//! Types for Cyclone V devices.
//! Derived from https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/index_frames.html
//!
//! These types are used to access the FPGA Manager registers and other Cyclone V specific
//! registers. The code should be platform-agnostic to allow for tests. This means no
//! assembly or architecture specific libraries.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

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

#[derive(Debug)]
pub struct SocFpga<M: MemoryMapper> {
    pub memory: M,
}

#[cfg(feature = "std")]
impl Default for SocFpga<memory::DevMemMemoryMapper> {
    fn default() -> Self {
        let memory =
            memory::DevMemMemoryMapper::create(addresses::SOCFPGA_BASE_ADDRESS, 0x0100_0000)
                .expect("Could not create memory mapper");

        Self::new(memory)
    }
}

impl<M: MemoryMapper> SocFpga<M> {
    pub fn new(memory: M) -> Self {
        Self { memory }
    }

    /// Returns the base of the memory.
    pub fn base(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    /// Returns the register as mutable.
    pub fn regs_mut(&mut self) -> &mut FpgaManagerRegs {
        unsafe {
            &mut *((self
                .memory
                .as_mut_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET))
                as *mut FpgaManagerRegs)
        }
    }

    /// Gets the Cyclone-V registers.
    pub fn regs(&self) -> &FpgaManagerRegs {
        unsafe {
            &*((self
                .memory
                .as_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET)) as *const FpgaManagerRegs)
        }
    }

    /// Get the DATA address.
    pub fn data(&self) -> *const u8 {
        unsafe {
            self.memory
                .as_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRDATA_OFFSET)
        }
    }

    /// Get the DATA address as mutable.
    pub fn data_mut(&mut self) -> *mut u8 {
        unsafe {
            self.memory
                .as_mut_ptr::<u8>()
                .add(offsets::SOCFPGA_FPGAMGRDATA_OFFSET)
        }
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

    #[inline]
    pub fn dclkcnt(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.dclkcnt)) }
    }

    #[inline]
    pub fn set_dclkcnt(&mut self, value: u32) {
        unsafe {
            write_volatile(addr_of_mut!(self.dclkcnt), value);
        }
    }

    #[inline]
    pub fn dclkstat(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.dclkstat)) }
    }

    #[inline]
    pub fn set_dclkstat(&mut self, value: u32) {
        unsafe {
            write_volatile(addr_of_mut!(self.dclkstat), value);
        }
    }

    /// Get the current set GPO register.
    #[inline]
    pub fn gpo(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.gpo)) }
    }

    #[inline]
    pub fn set_gpo(&mut self, value: u32) {
        unsafe {
            write_volatile(addr_of_mut!(self.gpo), value);
        }
    }

    /// Get the GPI register.
    #[inline]
    pub fn gpi(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.gpi)) }
    }

    #[inline]
    pub fn misci(&self) -> u32 {
        unsafe { read_volatile(addr_of!(self.misci)) }
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
}

#[test]
fn update_ctrl() {
    let mut region = vec![0; 0x1000000];
    let mapper = memory::RegionMemoryMapper::new(&mut region);
    let mut soc = SocFpga::new(mapper);

    let mgr = soc.regs_mut();

    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaConfigurationPins);
    mgr.update_ctrl(|ctrl| ctrl.set_en(ctrl::FpgaCtrlEn::FpgaManager));
    assert_eq!(mgr.ctrl().en(), ctrl::FpgaCtrlEn::FpgaManager);
}
