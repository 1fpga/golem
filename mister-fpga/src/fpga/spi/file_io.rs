use crate::fpga::feature::SpiFeature;
use crate::fpga::{IntoLowLevelSpiCommand, SpiCommand, SpiCommandExt};

#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
#[repr(u16)]
enum FileIoCommands {
    FileIoFileTx = 0x53,
    FileIoFileTxDat = 0x54,
    FileIoFileIndex = 0x55,
    FileIoFileInfo = 0x56,
}

impl IntoLowLevelSpiCommand for FileIoCommands {
    fn into_ll_spi_command(self) -> (SpiFeature, u16) {
        (SpiFeature::FPGA, self as u16)
    }
}

/// Sends the File Index to the FPGA.
pub struct FileIoFileIndex(u8);

impl SpiCommand for FileIoFileIndex {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(FileIoCommands::FileIoFileIndex).write_b(self.0);

        Ok(())
    }
}

impl From<u8> for FileIoFileIndex {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl FileIoFileIndex {
    #[inline]
    pub fn new(index: u8) -> Self {
        Self(index)
    }
}

/// Sends the File Info to the FPGA.
pub struct FileIoFileExtension<'a>(pub &'a str);

impl SpiCommand for FileIoFileExtension<'_> {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let ext_bytes = self.0.as_bytes();
        // Extend to 4 characters with the dot.
        let ext: [u8; 4] = [
            b'.',
            ext_bytes.get(0).copied().unwrap_or(0),
            ext_bytes.get(1).copied().unwrap_or(0),
            ext_bytes.get(2).copied().unwrap_or(0),
        ];

        spi.command(FileIoCommands::FileIoFileInfo)
            .write((ext[0] as u16) << 8 | ext[1] as u16)
            .write((ext[2] as u16) << 8 | ext[3] as u16);

        Ok(())
    }
}

/// Send the File size to the FPGA.
pub struct FileIoFileTxEnabled(pub Option<u32>);

impl SpiCommand for FileIoFileTxEnabled {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        let mut command = spi.command(FileIoCommands::FileIoFileTx);
        command.write_b(0xff);

        if let Some(size) = self.0 {
            command.write(size as u16).write((size >> 16) as u16);
        }

        Ok(())
    }
}

/// Send the File size to the FPGA.
pub struct FileIoFileTxDisabled;

impl SpiCommand for FileIoFileTxDisabled {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(FileIoCommands::FileIoFileTx).write_b(0);

        Ok(())
    }
}

/// Send the File data to the FPGA on a 8 bits bus.
pub struct FileIoFileTxData8Bits<'a>(pub &'a [u8]);

impl SpiCommand for FileIoFileTxData8Bits<'_> {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(FileIoCommands::FileIoFileTxDat)
            .write_buffer_b(self.0);
        Ok(())
    }
}

/// Send the File data to the FPGA on a 16 bits bus.
pub struct FileIoFileTxData16Bits<'a>(pub &'a [u16]);

impl SpiCommand for FileIoFileTxData16Bits<'_> {
    #[inline]
    fn execute<S: SpiCommandExt>(&mut self, spi: &mut S) -> Result<(), String> {
        spi.command(FileIoCommands::FileIoFileTxDat)
            .write_buffer(self.0);
        Ok(())
    }
}
