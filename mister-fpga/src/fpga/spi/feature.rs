use super::{SSPI_FPGA_FEATURE, SSPI_IO_FEATURE, SSPI_OSD_FEATURE};
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

/// Features to enable on the SPI bus.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct SpiFeature(u32);

impl Debug for SpiFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_tuple("SpiFeature");
        if self.fpga() {
            d.field(&"fpga");
        }
        if self.osd() {
            d.field(&"osd");
        }
        if self.io() {
            d.field(&"io");
        }
        d.finish()
    }
}

impl From<SpiFeature> for u32 {
    fn from(value: SpiFeature) -> Self {
        value.0
    }
}

impl From<u32> for SpiFeature {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Add<SpiFeature> for SpiFeature {
    type Output = SpiFeature;

    fn add(mut self, rhs: SpiFeature) -> Self::Output {
        self.0 = self.0 | rhs.0;
        self
    }
}

impl AddAssign<SpiFeature> for SpiFeature {
    fn add_assign(&mut self, rhs: SpiFeature) {
        self.0 = self.0 | rhs.0;
    }
}

impl SpiFeature {
    pub const FPGA: Self = Self(0).with_fpga();
    pub const OSD: Self = Self(0).with_osd();
    pub const IO: Self = Self(0).with_io();
    pub const ALL: Self = Self(0).with_fpga().with_osd().with_io();

    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    pub const fn fpga(&self) -> bool {
        self.0 & SSPI_FPGA_FEATURE != 0
    }

    pub const fn osd(&self) -> bool {
        self.0 & SSPI_OSD_FEATURE != 0
    }

    pub const fn io(&self) -> bool {
        self.0 & SSPI_IO_FEATURE != 0
    }

    pub fn set_fpga(&mut self, value: bool) {
        self.0 = self.0 & !SSPI_FPGA_FEATURE | ((value as u32) << 18);
    }

    pub fn set_osd(&mut self, value: bool) {
        self.0 = self.0 & !SSPI_OSD_FEATURE | ((value as u32) << 19);
    }

    pub fn set_io(&mut self, value: bool) {
        self.0 = self.0 & !SSPI_IO_FEATURE | ((value as u32) << 20);
    }

    pub const fn with_fpga(mut self) -> Self {
        self.0 = self.0 | SSPI_FPGA_FEATURE;
        self
    }
    pub const fn with_osd(mut self) -> Self {
        self.0 = self.0 | SSPI_OSD_FEATURE;
        self
    }
    pub const fn with_io(mut self) -> Self {
        self.0 = self.0 | SSPI_IO_FEATURE;
        self
    }
}
