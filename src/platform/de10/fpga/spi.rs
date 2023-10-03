use cyclone_v::memory::MemoryMapper;
use cyclone_v::SocFpga;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::rc::Rc;

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

impl SpiFeature {
    pub const FPGA: Self = Self(0).with_fpga();
    pub const OSD: Self = Self(0).with_osd();
    pub const IO: Self = Self(0).with_io();
    pub const ALL: Self = Self(0).with_fpga().with_osd().with_io();

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

#[derive(Debug)]
pub struct Spi<M: MemoryMapper> {
    soc: Rc<UnsafeCell<SocFpga<M>>>,
}

impl<M: MemoryMapper> Clone for Spi<M> {
    fn clone(&self) -> Self {
        Self {
            soc: self.soc.clone(),
        }
    }
}

impl<M: MemoryMapper> Spi<M> {
    pub fn new(soc: Rc<UnsafeCell<SocFpga<M>>>) -> Self {
        Self { soc }
    }

    #[inline]
    pub fn soc_mut(&mut self) -> &mut SocFpga<M> {
        unsafe { &mut *self.soc.get() }
    }

    // TODO: remove this.
    #[inline]
    pub fn enable_u32(&mut self, mask: u32) {
        let regs = self.soc_mut().regs_mut();

        let gpo = (regs.gpo() & SpiFeature::ALL.0) | 0x8000_0000;
        regs.set_gpo(gpo | mask);
    }

    // TODO: remove this.
    #[inline]
    pub fn disable_u32(&mut self, mask: u32) {
        let regs = self.soc_mut().regs_mut();

        let gpo: u32 = (regs.gpo() & SpiFeature::ALL.0) | 0x8000_0000;
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
    pub fn write_b(&mut self, byte: u8) -> u8 {
        self.write(byte) as u8
    }

    #[inline]
    pub fn write_block_16(&mut self, buffer: &[u16]) {
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

impl<M: MemoryMapper> std::io::Write for Spi<M> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let regs = self.soc_mut().regs_mut();
        let gpo_h = (regs.gpo() & !(SSPI_DATA_MASK | SSPI_STROBE)) | 0x8000_0000;
        let mut gpo = gpo_h;

        buf.iter().for_each(|b| {
            gpo = gpo_h | (*b as u32);
            regs.set_gpo(gpo);
            regs.set_gpo(gpo | SSPI_STROBE);
        });
        regs.set_gpo(gpo);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
