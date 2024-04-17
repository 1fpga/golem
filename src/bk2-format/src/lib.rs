use std::io::{Read, Seek};

pub use error::Bk2Error;

mod error;
mod header;

pub struct Bk2File<R: Read + Seek> {
    file: zip::ZipArchive<R>,
}

impl<R: Read + Seek> TryFrom<R> for Bk2File<R> {
    type Error = Bk2Error;

    fn try_from(file: R) -> Result<Self, Self::Error> {
        Ok(Self {
            file: zip::ZipArchive::new(file)?,
        })
    }
}

impl<R: Read + Seek> Bk2File<R> {
    pub fn load(file: R) -> Result<Self, Bk2Error> {
        file.try_into()
    }
}

