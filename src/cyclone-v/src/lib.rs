#![allow(clippy::missing_safety_doc)]
//! Types for Cyclone V devices.
//! Derived from https://www.intel.com/content/www/us/en/programmable/hps/cyclone-v/index_frames.html
//!
//! These types are used to access the FPGA Manager registers and other Cyclone V specific
//! registers. The code should be platform-agnostic to allow for tests. This means no
//! assembly or architecture specific libraries.
#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

use crate::memory::MemoryMapper;

mod macros;

pub mod fpgamgrregs;
pub mod l3regs;
pub mod memory;
pub mod rstmgr;
pub mod sdr;
pub mod sysmgr;

macro_rules! declare_field {
    ($(#[$field_attr:meta])* $field_name: ident, $const_name: ident, $ty: ty [pointer]) => {
        paste::paste! {
            $(#[$field_attr])*
            pub unsafe fn [< $field_name _ptr >](&self) -> *const $ty {
                self.memory.as_ptr::<u8>().add(offsets::$const_name) as *const $ty
            }

            $(#[$field_attr])*
            pub unsafe fn [< $field_name _ptr_mut >](&mut self) -> *mut $ty {
                self.memory.as_mut_ptr::<u8>().add(offsets::$const_name) as *mut $ty
            }

            $(#[$field_attr])*
            pub fn $field_name(&self) -> &$ty {
                unsafe {
                    &*(self.memory.as_ptr::<u8>().add(offsets::$const_name) as *const $ty)
                }
            }

            $(#[$field_attr])*
            pub fn [< $field_name _mut >](&mut self) -> &$ty {
                unsafe {
                    &mut *(self.memory.as_mut_ptr::<u8>().add(offsets::$const_name) as *mut $ty)
                }
            }
        }
    };
    ($(#[$field_attr:meta])* $field_name: ident, $const_name: ident, $ty: ty []) => {
        paste::paste! {
            $(#[$field_attr])*
            pub fn $field_name(&self) -> &$ty {
                unsafe {
                    &*((self.memory.as_ptr::<u8>().add(offsets::$const_name)) as *const $ty)
                }
            }

            $(#[$field_attr])*
            pub fn [< $field_name _mut >](&mut self) -> &mut $ty {
                unsafe {
                    &mut *((self.memory.as_mut_ptr::<u8>().add(offsets::$const_name)) as *mut $ty)
                }
            }
        }
    };
}

// Create various constants for memory locations for Cyclone V.
macro_rules! create_memory_locations {
    ($(
        $(#[$field_attr:meta])*
        $field_name: ident ($const_name: ident): $ty: ty $([$($tags: ident)*])? => $start: literal .. $end: literal
    );* $(;)?) => {
        /// Absolute addresses for the SoC FPGA.
        #[allow(dead_code)]
        mod addresses {
            $(
                pub const $const_name: usize = $start;
            )*
        }

        /// A list of offsets from the base address.
        #[allow(dead_code)]
        mod offsets {
            $(
                pub const $const_name: usize = (($start as usize) .saturating_sub(crate::addresses::BASE));
            )*
        }

        /// A list of sizes for the memory locations.
        #[allow(dead_code)]
        mod sizes {
            $(
                pub const $const_name: usize = ($end - $start);
            )*
        }

        /// Ranges of memory.
        pub mod ranges {
            $(
                pub const $const_name: core::ops::Range<usize> = $start .. $end;
            )*
        }

        #[derive(Debug, Copy, Clone)]
        pub struct SocFpga<M: memory::MemoryMapper> {
            pub memory: M,
        }

        impl<M: memory::MemoryMapper> SocFpga<M> {
            pub fn new(memory: M) -> Self {
                Self { memory }
            }

            $(
                declare_field!($(#[$field_attr])* $field_name, $const_name, $ty [$($($tags)*)?]);
            )*
        }
    };
}

create_memory_locations! {
    /// The base location of the memory pointed to with the memory mapper.
    base(BASE):                 u8 [pointer]                    => 0xFF000000 .. 0xFFFFFFFF;

    /// The accessible host memory from and to the core.
    host_memory(HOST_MEMORY):   u8 [pointer]                    => 0x20000000 .. 0x3FFFFFFF;

    /// The FPGA Manager registers.
    regs(FPGAMGRREGS):          fpgamgrregs::FpgaManagerRegs    => 0xFF706000 .. 0xFF706FFF;

    /// The FPGA Manager data.
    data(FPGAMGRDATA):          u8 [pointer]                    => 0xFFB90000 .. 0xFFB90003;

    /// The SDRAM Controller.
    sdr(SDR):                   sdr::SdramCtrl                  => 0xFFC20000 .. 0xFFC2FFFF;

    /// The Reset Manager.
    rstmgr(RSTMGR):             rstmgr::ResetManager            => 0xFFD05000 .. 0xFFD050FF;

    /// Registers to control L3 interconnect settings.
    l3regs(L3_REGS):            l3regs::L3Regs                  => 0xFF800000 .. 0xFF87FFFF;

    /// System Manager Module
    sysmgr(SYSMGR):             sysmgr::SystemManagerModule     => 0xFFD08000 .. 0xFFD08FFF;
}

// Safety: [`SocFpga`] is safe to send across threads if its memory mapper is safe as well.
unsafe impl<T: MemoryMapper + Send> Send for SocFpga<T> {}

#[cfg(feature = "std")]
impl Default for SocFpga<memory::DevMemMemoryMapper> {
    fn default() -> Self {
        let memory = memory::DevMemMemoryMapper::create(addresses::BASE, sizes::BASE)
            .expect("Could not create memory mapper");

        Self::new(memory)
    }
}

#[cfg(feature = "std")]
impl SocFpga<memory::BufferMemoryMapper> {
    pub fn create_for_test() -> Self {
        let memory = memory::BufferMemoryMapper::new(sizes::BASE);

        Self::new(memory)
    }
}
