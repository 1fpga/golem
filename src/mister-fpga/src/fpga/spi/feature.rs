use fixed_map::Key;
use std::fmt::Debug;
use std::ops::{BitOr, BitOrAssign};

/// A single SPI feature.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Key)]
#[repr(u32)]
pub enum SpiFeature {
    #[default]
    None = 0,
    Fpga = 1 << 18,
    Osd = 1 << 19,
    Io = 1 << 20,
}

impl SpiFeature {
    pub const fn as_u32(&self) -> u32 {
        *self as u32
    }
}

/// A set of features to enable/disable on the SPI bus.
#[derive(Clone, Copy, PartialEq)]
pub struct SpiFeatureSet(u32);

impl Default for SpiFeatureSet {
    fn default() -> Self {
        Self::NONE
    }
}

impl Debug for SpiFeatureSet {
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

impl From<SpiFeatureSet> for u32 {
    fn from(value: SpiFeatureSet) -> Self {
        value.0
    }
}

impl From<u32> for SpiFeatureSet {
    fn from(value: u32) -> Self {
        Self(value & Self::ALL.0)
    }
}

impl BitOr<SpiFeatureSet> for SpiFeatureSet {
    type Output = SpiFeatureSet;

    fn bitor(mut self, rhs: SpiFeatureSet) -> Self::Output {
        self.0 |= rhs.0;
        self
    }
}

impl BitOr<SpiFeature> for SpiFeatureSet {
    type Output = SpiFeatureSet;

    fn bitor(mut self, rhs: SpiFeature) -> Self::Output {
        self.0 |= rhs as u32;
        self
    }
}

impl BitOrAssign<SpiFeatureSet> for SpiFeatureSet {
    fn bitor_assign(&mut self, rhs: SpiFeatureSet) {
        self.0 |= rhs.0;
    }
}

impl BitOrAssign<SpiFeature> for SpiFeatureSet {
    fn bitor_assign(&mut self, rhs: SpiFeature) {
        self.0 |= rhs as u32;
    }
}

impl SpiFeatureSet {
    pub const NONE: Self = Self(0);
    pub const FPGA: Self = Self::NONE.with_fpga();
    pub const OSD: Self = Self::NONE.with_osd();
    pub const IO: Self = Self::NONE.with_io();
    pub const ALL: Self = Self::NONE.with_fpga().with_osd().with_io();

    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    pub const fn fpga(&self) -> bool {
        self.0 & SpiFeature::Fpga as u32 != 0
    }

    pub const fn osd(&self) -> bool {
        self.0 & SpiFeature::Osd as u32 != 0
    }

    pub const fn io(&self) -> bool {
        self.0 & SpiFeature::Io as u32 != 0
    }

    pub const fn is(&self, feature: SpiFeature) -> bool {
        self.0 & feature as u32 != 0
    }

    pub fn iter(&self) -> impl Iterator<Item = SpiFeature> {
        let mut fpga = self.fpga();
        let mut osd = self.osd();
        let mut io = self.io();

        std::iter::from_fn(move || {
            if fpga {
                fpga = false;
                Some(SpiFeature::Fpga)
            } else if osd {
                osd = false;
                Some(SpiFeature::Osd)
            } else if io {
                io = false;
                Some(SpiFeature::Io)
            } else {
                None
            }
        })
    }

    pub fn set_fpga(&mut self, value: bool) {
        self.0 = self.0 & !(SpiFeature::Fpga as u32) | ((value as u32) << 18);
    }

    pub fn set_osd(&mut self, value: bool) {
        self.0 = self.0 & !(SpiFeature::Osd as u32) | ((value as u32) << 19);
    }

    pub fn set_io(&mut self, value: bool) {
        self.0 = self.0 & !(SpiFeature::Io as u32) | ((value as u32) << 20);
    }

    pub fn set(&mut self, feature: SpiFeature, value: bool) {
        if value {
            self.0 |= feature as u32;
        } else {
            self.0 &= !(feature as u32);
        }
    }

    pub const fn with_fpga(self) -> Self {
        Self(self.0 | SpiFeature::Fpga as u32)
    }
    pub const fn with_osd(self) -> Self {
        Self(self.0 | SpiFeature::Osd as u32)
    }
    pub const fn with_io(self) -> Self {
        Self(self.0 | SpiFeature::Io as u32)
    }
}
