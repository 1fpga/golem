use crate::fpga::feature::SpiFeature;
use crate::fpga::{IntoLowLevelSpiCommand, SpiCommand, SpiCommandExt};

/// OSD SPI commands.
#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
#[repr(u16)]
enum OsdCommands {
    /// Write a line to the OSD. Lines are from `0..=8`.
    /// This goes to OsdWriteLine8 = 0x28.
    OsdWriteLine(u8) = 0x20,

    /// Disable the OSD menu.
    OsdDisable = 0x40,

    /// Enable the OSD menu.
    OsdEnable = 0x41,
}

impl IntoLowLevelSpiCommand for OsdCommands {
    #[inline]
    fn into_ll_spi_command(self) -> (SpiFeature, u16) {
        (
            SpiFeature::OSD,
            match self {
                OsdCommands::OsdWriteLine(a) => 0x20 + a as u16,
                OsdCommands::OsdDisable => 0x40,
                OsdCommands::OsdEnable => 0x41,
            },
        )
    }
}

pub struct OsdIoWriteLine<'a>(pub u8, pub &'a [u8]);

impl SpiCommand for OsdIoWriteLine<'_> {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(OsdCommands::OsdWriteLine(self.0))
            .write_buffer_b(self.1);

        Ok(())
    }
}

pub struct OsdEnable;

impl SpiCommand for OsdEnable {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(OsdCommands::OsdEnable);
        Ok(())
    }
}

pub struct OsdDisable;

impl SpiCommand for OsdDisable {
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(OsdCommands::OsdDisable);
        Ok(())
    }
}
