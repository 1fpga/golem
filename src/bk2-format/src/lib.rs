use std::io::{Read, Seek};
use std::path::Path;

mod error;
mod header;

pub use error::Bk2Error;

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

impl Bk2File<_> {
    pub fn header(&self) -> Result<Bk2Header>
}
