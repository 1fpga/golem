use crate::fpga::feature::{SpiFeature, SpiFeatureSet};
use cyclone_v::memory::MemoryMapper;
use cyclone_v::SocFpga;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::trace;

/// SPI is a 16-bit data bus where the lowest 16 bits are the data and the highest 16-bits
/// are the control bits.
const SSPI_DATA_MASK: u32 = 0x0000_FFFF;

/// This signal is sent to indicate new data.
const SSPI_STROBE: u32 = 1 << 17;
/// This signal is received to indicate that the data was read.
const SSPI_ACK: u32 = 1 << 17;

pub mod feature;
pub mod file_io;
pub mod osd_io;
pub mod user_io;

pub trait SpiCommandExt: Sized {
    fn command(&mut self, command: impl IntoLowLevelSpiCommand) -> SpiCommandGuard<Self> {
        self.command_read(command, &mut 0)
    }
    fn command_read(
        &mut self,
        command: impl IntoLowLevelSpiCommand,
        out: &mut u16,
    ) -> SpiCommandGuard<Self>;
    fn write(&mut self, word: u16) -> &mut Self;
    fn write_read(&mut self, word: u16, out: &mut u16) -> &mut Self;
    fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self;
    fn write_cond(&mut self, cond: bool, word: u16) -> &mut Self;
    fn write_cond_b(&mut self, cond: bool, word: u8) -> &mut Self;
    fn write_buffer(&mut self, buffer: &[u16]) -> &mut Self;
    fn write_buffer_b(&mut self, buffer: &[u8]) -> &mut Self;
    fn write_b(&mut self, byte: u8) -> &mut Self;
    fn enable(&mut self, feature: SpiFeatureSet) -> &mut Self;
    fn disable(&mut self, feature: SpiFeatureSet) -> &mut Self;
}

pub trait IntoLowLevelSpiCommand {
    fn into_ll_spi_command(self) -> (SpiFeatureSet, u16);
}

pub trait SpiCommand {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String>;
}

pub struct SpiCommandGuard<'a, S: SpiCommandExt> {
    spi: &'a mut S,
    feature: SpiFeatureSet,
}

impl<'a, S: SpiCommandExt> SpiCommandGuard<'a, S> {
    #[inline]
    pub fn new(spi: &'a mut S, feature: SpiFeatureSet) -> Self {
        Self { spi, feature }
    }

    pub fn execute(&mut self, mut command: impl SpiCommand) -> Result<(), String> {
        command.execute(self.spi)
    }

    #[inline]
    pub fn write(&mut self, word: u16) -> &mut Self {
        self.spi.write(word);
        self
    }

    #[inline]
    pub fn write_nz(&mut self, word: u16) -> &mut Self {
        let word = word;
        if word != 0 {
            self.spi.write(word);
        }
        self
    }

    #[inline]
    pub fn write_read(&mut self, word: u16, out: &mut u16) -> &mut Self {
        self.spi.write_read(word, out);
        self
    }

    #[inline]
    pub fn write_read_32(&mut self, word1: u16, word2: u16, out: &mut u32) -> &mut Self {
        let mut high: u16 = 0;
        let mut low: u16 = 0;
        self.write_read(word1, &mut low)
            .write_read(word2, &mut high);
        *out = (high as u32) << 16 | (low as u32);
        self
    }

    #[inline]
    pub fn write_get(&mut self, word: u16) -> u16 {
        let mut out = 0;
        self.spi.write_read(word, &mut out);
        out
    }

    #[inline]
    pub fn write_get_32(&mut self, word1: u16, word2: u16) -> u32 {
        let mut out = 0;
        self.write_read_32(word1, word2, &mut out);
        out
    }

    #[inline]
    pub fn get(&mut self) -> u16 {
        self.write_get(0)
    }

    #[inline]
    pub fn get_32(&mut self) -> u32 {
        self.write_get_32(0, 0)
    }

    #[inline]
    pub fn write_cond(&mut self, cond: bool, word: u16) -> &mut Self {
        self.spi.write_cond(cond, word);
        self
    }

    #[inline]
    pub fn write_cond_b(&mut self, cond: bool, word: u8) -> &mut Self {
        self.spi.write_cond_b(cond, word);
        self
    }

    #[inline]
    pub fn write_buffer_w(&mut self, buffer: &[u16]) -> &mut Self {
        for word in buffer {
            self.spi.write(*word);
        }
        self
    }

    #[inline]
    pub fn write_buffer_cond_w(&mut self, cond: bool, buffer: &[u16]) -> &mut Self {
        if cond {
            for word in buffer {
                self.spi.write(*word);
            }
        }
        self
    }

    #[inline]
    pub fn write_buffer_b(&mut self, buffer: &[u8]) -> &mut Self {
        for word in buffer {
            self.spi.write_b(*word);
        }
        self
    }

    #[inline]
    pub fn write_32(&mut self, word: u32) -> &mut Self {
        self.spi.write(word as u16);
        self.spi.write((word >> 16) as u16);
        self
    }

    #[inline]
    pub fn write_b(&mut self, byte: u8) -> &mut Self {
        self.write(byte as u16)
    }

    #[inline]
    pub fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self {
        self.spi.write_read_b(byte, out);
        self
    }

    #[inline]
    pub fn read_buffer_w(&mut self, buffer: &mut [u16]) -> &mut Self {
        for word in buffer {
            self.spi.write_read(0u16, word);
        }
        self
    }

    #[inline]
    pub fn read_buffer_b(&mut self, buffer: &mut [u8]) -> &mut Self {
        for word in buffer {
            self.spi.write_read_b(0u8, word);
        }
        self
    }
}

impl<'a, S: SpiCommandExt> Drop for SpiCommandGuard<'a, S> {
    fn drop(&mut self) {
        self.spi.disable(self.feature);
    }
}

#[derive(Debug)]
pub struct Spi<M: MemoryMapper> {
    soc: Arc<UnsafeCell<SocFpga<M>>>,

    // Ref counting features to prevent double enable (performance) and double
    // disable (error). We only actually enable if the refcount is 0, and disable
    // if the refcount is 1.
    features: fixed_map::Map<SpiFeature, u32>,
}
unsafe impl<M: MemoryMapper> Send for Spi<M> {}
unsafe impl<M: MemoryMapper> Sync for Spi<M> {}

impl<M: MemoryMapper> Clone for Spi<M> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            soc: self.soc.clone(),
            features: self.features,
        }
    }
}

impl<M: MemoryMapper> Spi<M> {
    pub fn new(soc: Arc<UnsafeCell<SocFpga<M>>>) -> Self {
        Self {
            soc,
            features: Default::default(),
        }
    }

    #[inline]
    pub fn soc_mut(&mut self) -> &mut SocFpga<M> {
        unsafe { &mut *self.soc.get() }
    }

    #[inline]
    pub fn execute(&mut self, mut command: impl SpiCommand) -> Result<(), String> {
        command.execute(self)
    }

    #[inline]
    pub(super) fn enable_u32(&mut self, mask: u32) {
        let mut new_mask = 0;
        SpiFeatureSet::from(mask).iter().for_each(|feature| {
            let count = self.features.entry(feature).or_default();
            if *count == 0 {
                new_mask |= feature.as_u32();
            }
            *count += 1;
        });
        if new_mask == 0 {
            return;
        }

        let regs = self.soc_mut().regs_mut();
        let gpo = (regs.gpo() & SpiFeatureSet::ALL.as_u32()) | 0x8000_0000;
        regs.set_gpo(gpo | new_mask);
    }

    #[inline]
    pub(super) fn disable_u32(&mut self, mask: u32) {
        let mut new_mask = 0;
        SpiFeatureSet::from(mask).iter().for_each(|feature| {
            let count = self.features.entry(feature).or_default();
            if *count == 1 {
                new_mask |= feature.as_u32();
                *count = 0;
            } else if *count == 0 {
                trace!("double disable {:?}", feature);
            } else {
                *count -= 1;
            }
        });
        if new_mask == 0 {
            return;
        }

        let regs = self.soc_mut().regs_mut();
        let gpo: u32 = (regs.gpo() & SpiFeatureSet::ALL.as_u32()) | 0x8000_0000;
        regs.set_gpo(gpo & !new_mask);
    }

    #[inline]
    pub fn enable(&mut self, feature: SpiFeatureSet) {
        self.enable_u32(feature.into());
    }

    #[inline]
    pub fn disable(&mut self, feature: SpiFeatureSet) {
        self.disable_u32(feature.into());
    }

    /// Gets the config string of the core.
    /// This method should only be called once per core, ideally when the core boot up.
    #[inline]
    pub fn config_string(&mut self) -> String {
        let mut str_builder = String::with_capacity(10240);
        self.execute(user_io::UserIoGetString(&mut str_builder))
            .unwrap();
        str_builder
    }

    #[inline]
    fn inner_command(
        &mut self,
        command: impl IntoLowLevelSpiCommand,
        out: &mut u16,
    ) -> SpiFeatureSet {
        let (feature, command) = command.into_ll_spi_command();
        self.enable(feature);
        *out = self.write(command);
        feature
    }

    #[inline]
    #[must_use]
    pub fn command(&mut self, command: impl IntoLowLevelSpiCommand) -> SpiCommandGuard<'_, Self> {
        let feature = self.inner_command(command, &mut 0);

        SpiCommandGuard::new(self, feature)
    }

    #[inline]
    #[must_use]
    pub fn command_read(
        &mut self,
        command: impl IntoLowLevelSpiCommand,
        out: &mut u16,
    ) -> SpiCommandGuard<'_, Self> {
        let feature = self.inner_command(command, out);

        SpiCommandGuard::new(self, feature)
    }

    /// Send a 16-bit word to the core. Returns the 16-bit word received from the core.
    #[inline]
    pub fn write(&mut self, word: u16) -> u16 {
        let regs = self.soc_mut().regs_mut();

        // Remove the strobe bit and set the data bits.
        let gpo = (regs.gpo() & !(SSPI_DATA_MASK | SSPI_STROBE)) | (word as u32);

        regs.set_gpo(gpo);
        regs.set_gpo(gpo | SSPI_STROBE);

        // Wait for the ACK bit to be unset to give time to the core to get some work.
        loop {
            let gpi = regs.gpi();
            if gpi & SSPI_ACK != 0 {
                break;
            }
        }

        // Send the actual data without the strobe, then wait for the core to get done.
        regs.set_gpo(gpo);
        loop {
            let gpi = regs.gpi();
            if gpi & SSPI_ACK == 0 {
                break gpi as u16;
            }
        }
    }

    #[inline]
    pub fn write_block_8(&mut self, buffer: &[u8]) -> Result<usize, String> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let regs = self.soc_mut().regs_mut();
        let gpo_h = (regs.gpo() & !(SSPI_DATA_MASK | SSPI_STROBE)) | 0x8000_0000;
        let mut gpo = gpo_h;

        buffer.iter().for_each(|b| {
            gpo = gpo_h | (*b as u32);
            regs.set_gpo(gpo);
            regs.set_gpo(gpo | SSPI_STROBE);
        });
        regs.set_gpo(gpo);

        Ok(buffer.len())
    }

    #[inline]
    pub fn write_b(&mut self, byte: u8) -> u8 {
        self.write(byte as u16) as u8
    }

    #[inline]
    pub fn write_block_16(&mut self, buffer: &[u16]) {
        if buffer.is_empty() {
            return;
        }

        let regs = self.soc_mut().regs_mut();
        let gpo_h = regs.gpo() & !(SSPI_DATA_MASK | SSPI_STROBE);
        let mut gpo = gpo_h;

        buffer.iter().for_each(|b| {
            gpo = gpo_h | (*b as u32);
            regs.set_gpo(gpo);
            regs.set_gpo(gpo | SSPI_STROBE);
        });
        regs.set_gpo(gpo);
    }
}

impl<M: MemoryMapper> SpiCommandExt for Spi<M> {
    #[inline]
    fn command_read(
        &mut self,
        command: impl IntoLowLevelSpiCommand,
        out: &mut u16,
    ) -> SpiCommandGuard<Self> {
        self.command_read(command, out)
    }

    #[inline]
    fn write(&mut self, word: u16) -> &mut Self {
        self.write(word);
        self
    }

    #[inline]
    fn write_read(&mut self, word: u16, out: &mut u16) -> &mut Self {
        *out = self.write(word);
        self
    }

    #[inline]
    fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self {
        *out = self.write_b(byte);
        self
    }

    #[inline]
    fn write_cond(&mut self, cond: bool, word: u16) -> &mut Self {
        if cond {
            self.write(word);
        }
        self
    }

    #[inline]
    fn write_cond_b(&mut self, cond: bool, byte: u8) -> &mut Self {
        if cond {
            self.write_b(byte);
        }
        self
    }

    #[inline]
    fn write_buffer(&mut self, buffer: &[u16]) -> &mut Self {
        self.write_block_16(buffer);
        self
    }

    #[inline]
    fn write_buffer_b(&mut self, buffer: &[u8]) -> &mut Self {
        for word in buffer {
            self.write_b(*word);
        }
        self
    }

    #[inline]
    fn write_b(&mut self, byte: u8) -> &mut Self {
        self.write(byte as u16);
        self
    }

    #[inline]
    fn enable(&mut self, feature: SpiFeatureSet) -> &mut Self {
        self.enable(feature);
        self
    }

    #[inline]
    fn disable(&mut self, feature: SpiFeatureSet) -> &mut Self {
        self.disable(feature);
        self
    }
}

#[test]
pub fn features_refcount() {
    let soc = SocFpga::create_for_test();
    let mut spi = Spi::new(Arc::new(UnsafeCell::new(soc)));

    spi.enable(SpiFeatureSet::FPGA);
    spi.enable(SpiFeatureSet::FPGA);
    spi.disable(SpiFeatureSet::FPGA);
    spi.disable(SpiFeatureSet::IO);

    assert_eq!(spi.features.get(SpiFeature::Fpga), Some(&1));
}
