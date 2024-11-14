use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

/// A BIOS, including any information the core needs to know about the BIOS.
#[derive(Debug, Clone)]
pub enum Bios {
    /// A BIOS that is stored in memory.
    Memory(Option<PathBuf>, Cursor<Vec<u8>>),

    /// A BIOS that is stored in a file.
    File(PathBuf, Arc<std::fs::File>),
}

impl Read for Bios {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Memory(_, data) => data.read(buf),
            Self::File(_, file) => file.read(buf),
        }
    }
}

impl Seek for Bios {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Memory(_, data) => data.seek(pos),
            Self::File(_, file) => file.seek(pos),
        }
    }
}
