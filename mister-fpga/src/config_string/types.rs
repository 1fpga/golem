use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Formatter};
use std::num::NonZeroU32;

pub const FPGA_MEMORY_BASE: usize = 0x2000_0000;
pub const FPGA_MEMORY_SIZE: usize = 0x0200_0000;

/// A memory address in the RAM, that is accessible by the FPGA.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct FpgaRamMemoryAddress(NonZeroU32);

impl Debug for FpgaRamMemoryAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FpgaRamMemoryAddress")
            .field(&format_args!("0x{:08x}", self.as_u32()))
            .finish()
    }
}

impl FpgaRamMemoryAddress {
    pub fn as_u32(&self) -> u32 {
        self.0.get()
    }

    pub fn as_usize(&self) -> usize {
        self.0.get() as usize
    }
}

impl TryFrom<u32> for FpgaRamMemoryAddress {
    type Error = &'static str;

    fn try_from(inner: u32) -> Result<Self, Self::Error> {
        (inner as usize).try_into()
    }
}

impl TryFrom<usize> for FpgaRamMemoryAddress {
    type Error = &'static str;

    fn try_from(inner: usize) -> Result<Self, Self::Error> {
        if !cyclone_v::ranges::HOST_MEMORY.contains(&inner) {
            return Err("Address out of range.");
        }

        Ok(Self(NonZeroU32::new(inner as u32).unwrap()))
    }
}

impl From<FpgaRamMemoryAddress> for u32 {
    fn from(value: FpgaRamMemoryAddress) -> Self {
        value.as_u32()
    }
}
