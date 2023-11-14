use cyclone_v::memory::MemoryMapper;
use cyclone_v::SocFpga;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};
use std::rc::Rc;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SpiCommands {
    UserIoStatus = 0x00,
    UserIoButtonSwitch = 0x01,

    UserIoJoystick0 = 0x02,
    UserIoJoystick1 = 0x03,
    UserIoMouse = 0x04,
    UserIoKeyboard = 0x05,
    UserIoKeyboardOsd = 0x06,

    UserIoJoystick2 = 0x10,
    UserIoJoystick3 = 0x11,
    UserIoJoystick4 = 0x12,
    UserIoJoystick5 = 0x13,

    UserIoGetString = 0x14,

    SetStatus32Bits = 0x1e,

    /// Update status from the core
    GetStatusBits = 0x29,

    FileIoFileTx = 0x53,
    FileIoFileTxDat = 0x54,
    FileIoFileIndex = 0x55,
    FileIoFileInfo = 0x56,
}

impl SpiCommands {
    #[inline]
    pub fn from_joystick_index(index: u8) -> Self {
        match index {
            0 => Self::UserIoJoystick0,
            1 => Self::UserIoJoystick1,
            2 => Self::UserIoJoystick2,
            3 => Self::UserIoJoystick3,
            4 => Self::UserIoJoystick4,
            _ => Self::UserIoJoystick5,
        }
    }

    #[inline]
    pub fn is_user_io(&self) -> bool {
        matches!(self, Self::UserIoKeyboard)
    }

    #[inline]
    fn spi_feature(&self) -> SpiFeature {
        match self {
            Self::UserIoKeyboard
            | Self::UserIoMouse
            | Self::UserIoJoystick0
            | Self::UserIoJoystick1
            | Self::UserIoJoystick2
            | Self::UserIoJoystick3
            | Self::UserIoJoystick4
            | Self::UserIoJoystick5
            | Self::UserIoGetString
            | Self::SetStatus32Bits
            | Self::GetStatusBits => SpiFeature::IO,

            Self::FileIoFileIndex
            | Self::FileIoFileInfo
            | Self::FileIoFileTx
            | Self::FileIoFileTxDat => SpiFeature::FPGA,

            _ => SpiFeature::ALL,
        }
    }
}

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

pub struct SpiCommand<'a, M: MemoryMapper> {
    spi: &'a mut Spi<M>,
    feature: SpiFeature,
}

impl<'a, M: MemoryMapper> SpiCommand<'a, M> {
    #[inline]
    pub fn new(spi: &'a mut Spi<M>, feature: SpiFeature) -> Self {
        Self { spi, feature }
    }

    #[inline]
    pub fn enable(mut self, feature: SpiFeature) -> Self {
        self.feature += feature;
        self.spi.enable(self.feature);
        self
    }

    #[inline]
    pub fn write(self, word: impl Into<u16>) -> Self {
        self.spi.write(word);
        self
    }

    #[inline]
    pub fn write_read(self, word: impl Into<u16>, out: &mut u16) -> Self {
        *out = self.spi.write(word);
        self
    }

    #[inline]
    pub fn write_cond(self, cond: bool, word: impl Into<u16>) -> Self {
        if cond {
            self.spi.write(word);
        }
        self
    }

    #[inline]
    pub fn write_if(self, cond: impl FnOnce() -> bool, word: impl Into<u16>) -> Self {
        if cond() {
            self.spi.write(word);
        }
        self
    }

    #[inline]
    pub fn write_buffer(self, buffer: &[u16]) -> Self {
        for word in buffer {
            self.spi.write(*word);
        }
        self
    }

    #[inline]
    pub fn write_buffer_b(self, buffer: &[u8]) -> Self {
        for word in buffer {
            self.spi.write_b(*word);
        }
        self
    }

    #[inline]
    pub fn write_b(self, byte: u8) -> Self {
        self.write(byte)
    }

    #[inline]
    pub fn write_store(self, word: impl Into<u16>, store: &mut u16) -> Self {
        *store = self.spi.write(word);
        self
    }
}

impl<'a, M: MemoryMapper> Drop for SpiCommand<'a, M> {
    fn drop(&mut self) {
        self.spi.disable(self.feature);
    }
}

impl<'a, M: MemoryMapper> std::io::Write for SpiCommand<'a, M> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let regs = self.spi.soc_mut().regs_mut();
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

    /// Gets the config string of the core.
    /// This method should only be called once per core, ideally when the core boot up.
    #[inline]
    pub fn config_string(&mut self) -> String {
        let mut str_builder = String::with_capacity(10240);
        let feature = self.inner_command(SpiCommands::UserIoGetString, &mut 0);

        loop {
            let i = self.write_b(0);
            if i == 0 || i > 127 {
                break;
            }
            str_builder.push(i as char);
        }
        self.disable(feature);

        str_builder
    }

    #[inline]
    fn inner_command(&mut self, command: SpiCommands, out: &mut u16) -> SpiFeature {
        let feature = command.spi_feature();
        self.enable(feature);
        *out = self.write(command as u16);
        feature
    }

    #[inline]
    #[must_use]
    pub fn command(&mut self, command: SpiCommands) -> SpiCommand<'_, M> {
        let feature = self.inner_command(command, &mut 0);

        SpiCommand::new(self, feature)
    }

    #[inline]
    #[must_use]
    pub fn command_read(&mut self, command: SpiCommands, out: &mut u16) -> SpiCommand<'_, M> {
        let feature = self.inner_command(command, out);

        SpiCommand::new(self, feature)
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
