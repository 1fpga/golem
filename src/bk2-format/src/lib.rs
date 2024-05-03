use std::io::{Read, Seek};

pub use error::Bk2Error;

mod error;
mod header;

pub struct Bk2File<R: Read + Seek> {
    file: zip::ZipArchive<R>,
}

impl<R: Read + Seek> Bk2File<R> {
    pub fn load(file: R) -> Result<Self, Bk2Error> {
        todo!()
    }
}
