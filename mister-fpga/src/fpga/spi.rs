use crate::fpga::feature::SpiFeature;
use cyclone_v::memory::MemoryMapper;
use cyclone_v::SocFpga;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::sync::Arc;

/// SPI is a 16-bit data bus where the lowest 16 bits are the data and the highest 16-bits
/// are the control bits.
const SSPI_DATA_MASK: u32 = 0x0000_FFFF;

/// This signal is sent to indicate new data.
const SSPI_STROBE: u32 = 1 << 17;
/// This signal is received to indicate that the data was read.
const SSPI_ACK: u32 = 1 << 17;

/// Feature for FPGA.
const SSPI_FPGA_FEATURE: u32 = 1 << 18;

/// Feature for OSD.
const SSPI_OSD_FEATURE: u32 = 1 << 19;

/// Feature for IO.
const SSPI_IO_FEATURE: u32 = 1 << 20;

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
    fn write(&mut self, word: impl Into<u16>) -> &mut Self;
    fn write_read(&mut self, word: impl Into<u16>, out: &mut u16) -> &mut Self;
    fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self;
    fn write_cond(&mut self, cond: bool, word: impl Into<u16>) -> &mut Self;
    fn write_cond_b(&mut self, cond: bool, word: impl Into<u8>) -> &mut Self;
    fn write_buffer(&mut self, buffer: &[u16]) -> &mut Self;
    fn write_buffer_b(&mut self, buffer: &[u8]) -> &mut Self;
    fn write_b(&mut self, byte: u8) -> &mut Self;
    fn enable(&mut self, feature: SpiFeature) -> &mut Self;
    fn disable(&mut self, feature: SpiFeature) -> &mut Self;
}

pub trait IntoLowLevelSpiCommand {
    fn into_ll_spi_command(self) -> (SpiFeature, u16);
}

pub trait SpiCommand {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String>;
}

pub struct SpiCommandGuard<'a, S: SpiCommandExt> {
    spi: &'a mut S,
    feature: SpiFeature,
}

impl<'a, S: SpiCommandExt> SpiCommandGuard<'a, S> {
    #[inline]
    pub fn new(spi: &'a mut S, feature: SpiFeature) -> Self {
        Self { spi, feature }
    }

    #[inline]
    pub fn write(&mut self, word: impl Into<u16>) -> &mut Self {
        self.spi.write(word);
        self
    }

    #[inline]
    pub fn write_nz(&mut self, word: impl Into<u16>) -> &mut Self {
        let word = word.into();
        if word != 0 {
            self.spi.write(word);
        }
        self
    }

    #[inline]
    pub fn write_read(&mut self, word: impl Into<u16>, out: &mut u16) -> &mut Self {
        self.spi.write_read(word, out);
        self
    }

    #[inline]
    pub fn write_cond(&mut self, cond: bool, word: impl Into<u16>) -> &mut Self {
        self.spi.write_cond(cond, word);
        self
    }

    #[inline]
    pub fn write_cond_b(&mut self, cond: bool, word: impl Into<u8>) -> &mut Self {
        self.spi.write_cond_b(cond, word);
        self
    }

    #[inline]
    pub fn write_buffer(&mut self, buffer: &[u16]) -> &mut Self {
        for word in buffer {
            self.spi.write(*word);
        }
        self
    }

    #[inline]
    pub fn write_buffer_cond(&mut self, cond: bool, buffer: &[u16]) -> &mut Self {
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
    pub fn write_b(&mut self, byte: u8) -> &mut Self {
        self.write(byte)
    }

    #[inline]
    pub fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self {
        self.spi.write_read_b(byte, out);
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
}
unsafe impl<M: MemoryMapper> Send for Spi<M> {}
unsafe impl<M: MemoryMapper> Sync for Spi<M> {}

impl<M: MemoryMapper> Clone for Spi<M> {
    fn clone(&self) -> Self {
        Self {
            soc: self.soc.clone(),
        }
    }
}

impl<M: MemoryMapper> Spi<M> {
    pub fn new(soc: Arc<UnsafeCell<SocFpga<M>>>) -> Self {
        Self { soc }
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
        let regs = self.soc_mut().regs_mut();

        let gpo = (regs.gpo() & SpiFeature::ALL.as_u32()) | 0x8000_0000;
        regs.set_gpo(gpo | mask);
    }

    #[inline]
    pub(super) fn disable_u32(&mut self, mask: u32) {
        let regs = self.soc_mut().regs_mut();

        let gpo: u32 = (regs.gpo() & SpiFeature::ALL.as_u32()) | 0x8000_0000;
        regs.set_gpo(gpo & !mask);
    }

    #[inline]
    pub fn enable(&mut self, feature: SpiFeature) {
        self.enable_u32(feature.into());
    }

    #[inline]
    pub fn disable(&mut self, feature: SpiFeature) {
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
    fn inner_command(&mut self, command: impl IntoLowLevelSpiCommand, out: &mut u16) -> SpiFeature {
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
    pub fn write(&mut self, word: impl Into<u16>) -> u16 {
        let regs = self.soc_mut().regs_mut();

        // Remove the strobe bit and set the data bits.
        let gpo = (regs.gpo() & !(SSPI_DATA_MASK | SSPI_STROBE)) | (word.into() as u32);

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
        self.write(byte) as u8
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
    fn command_read(
        &mut self,
        command: impl IntoLowLevelSpiCommand,
        out: &mut u16,
    ) -> SpiCommandGuard<Self> {
        self.command_read(command, out)
    }

    #[inline]
    fn write(&mut self, word: impl Into<u16>) -> &mut Self {
        self.write(word);
        self
    }

    #[inline]
    fn write_read(&mut self, word: impl Into<u16>, out: &mut u16) -> &mut Self {
        *out = self.write(word);
        self
    }

    fn write_read_b(&mut self, byte: u8, out: &mut u8) -> &mut Self {
        *out = self.write_b(byte);
        self
    }

    #[inline]
    fn write_cond(&mut self, cond: bool, word: impl Into<u16>) -> &mut Self {
        if cond {
            self.write(word);
        }
        self
    }

    fn write_cond_b(&mut self, cond: bool, word: impl Into<u8>) -> &mut Self {
        if cond {
            self.write_b(word.into());
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
        self.write(byte);
        self
    }

    fn enable(&mut self, feature: SpiFeature) -> &mut Self {
        self.enable(feature);
        self
    }

    fn disable(&mut self, feature: SpiFeature) -> &mut Self {
        self.disable(feature);
        self
    }
}
