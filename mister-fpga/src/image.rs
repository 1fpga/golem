use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// An SdImage is a file that contains a device that can be mounted to
/// the core. For example, in console cores this could be the save data.
/// In a Commodore 64 core, this could be the disk image of a tape drive.
pub struct DiskImage {
    inner: DiskImageType,
}

enum DiskImageType {
    ReadOnly(Vec<u8>),
    ReadWrite(File),
    Growable(File),
}

impl DiskImage {
    pub fn readonly(mut reader: impl Read) -> Result<Self, std::io::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(Self {
            inner: DiskImageType::ReadOnly(data),
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;

        let readable = file.metadata()?.permissions().readonly();
        if readable {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            Self::readonly(data.as_slice())
        } else {
            Ok(Self {
                inner: DiskImageType::ReadWrite(file),
            })
        }
    }
}
