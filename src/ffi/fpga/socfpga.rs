//! Cyclone V SoC FPGA Structures.
//! Derived from https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/index_frames.html
use crate::shmem::Mapper;
use std::ptr::{addr_of_mut, NonNull};
use volatile::{VolatilePtr, VolatileRef};

/// Absolute addresses for the SoC FPGA.
mod addresses {
    pub const SOCFPGA_BASE_ADDRESS: usize = 0xFF000000;
    pub const SOCFPGA_FPGAMGRREGS_ADDRESS: usize = 0xFF706000;
}

/// A list of offsets from the base address.
mod offsets {
    use super::addresses::*;

    pub const SOCFPGA_FPGAMGRREGS_OFFSET: usize =
        SOCFPGA_FPGAMGRREGS_ADDRESS - SOCFPGA_BASE_ADDRESS;
}

#[derive(Debug)]
pub struct SocFpga {
    pub memory: Mapper,
}

extern "C" {
    static mut map_base: *mut u32;
}

impl Default for SocFpga {
    fn default() -> Self {
        let mut memory = Mapper::new(addresses::SOCFPGA_BASE_ADDRESS, 0x0100_0000);
        // TODO: remove this when the fpga code from CPP is gone.
        unsafe { map_base = memory.as_mut_ptr() as *mut u32 };

        Self { memory }
    }
}

impl SocFpga {
    fn fpga_manager_mut_ptr(&mut self) -> NonNull<FpgaManager> {
        unsafe {
            NonNull::new(
                (self
                    .memory
                    .as_mut_ptr()
                    .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET)) as *mut FpgaManager,
            )
            .unwrap()
        }
    }

    pub fn fpga_manager_mut(&mut self) -> &mut FpgaManager {
        unsafe {
            &mut *((self
                .memory
                .as_mut_ptr()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET)) as *mut FpgaManager)
        }
    }
    pub fn fpga_manager(&self) -> &FpgaManager {
        unsafe {
            &*((self
                .memory
                .as_ptr()
                .add(offsets::SOCFPGA_FPGAMGRREGS_OFFSET)) as *const FpgaManager)
        }
    }
}

/// The FPGA Manager is a hardware block that manages the FPGA configuration
/// process. It is responsible for configuring the FPGA with the image stored
/// in the flash memory. It also provides a mechanism to reconfigure the FPGA
/// with a new image stored in the flash memory.
///
/// This structure must be aligned exactly with the hardware registers.
///
/// See https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/sfo1410067808053.html#sfo1410067808053
#[repr(C)]
pub struct FpgaManager {
    // FPGA Manager Module
    /// Status Register
    pub stat: u32, // 0x00
    /// Control Register
    pub ctrl: u32,
    /// Data Clock Count Register
    pub dclkcnt: u32,
    /// Data Clock Status Register
    pub dclkstat: u32,
    /// General Purpose Output Register
    pub gpo: u32,
    /// General Purpose Input Register (RO)
    pub gpi: u32,
    /// Miscellaneous Input Register (RO)
    pub misci: u32,

    // Padding
    _pad_0x1c_0x82c: [u32; 517],

    // Configuration Monitor (MON) Registers
    /// GPIO Interrupt Enable Register
    pub gpio_inten: u32,
    /// GPIO Interrupt Mask Register
    pub gpio_intmask: u32,
    /// GPIO Interrupt Type Level Register
    pub gpio_inttype_level: u32,
    /// GPIO Interrupt Polarity Register
    pub gpio_int_polarity: u32,
    /// GPIO Interrupt Status Register
    pub gpio_intstatus: u32,
    /// GPIO Raw Interrupt Status Register
    pub gpio_raw_intstatus: u32,

    // Padding
    _pad_0x848: u32,

    /// GPIO Port A Clear Interrupt Register
    pub gpio_porta_eoi: u32,
    /// GPIO Port A External Port Register
    pub gpio_ext_porta: u32,

    // Padding
    _pad_0x854_0x85c: [u32; 3],

    /// GPIO Level Synchronization Register
    pub gpio_ls_sync: u32,

    // Padding
    _pad_0x864_0x868: [u32; 2],

    /// GPIO Version ID Code Register
    pub gpio_ver_id_code: u32,
    /// GPIO Configuration Register 2
    pub gpio_config_reg2: u32,
    /// GPIO Configuration Register 1
    pub gpio_config_reg1: u32,
}

impl FpgaManager {
    pub fn set_gpo(&mut self, value: u32) {
        unsafe {
            addr_of_mut!(self.gpo).write_volatile(value);
        }
    }
}
