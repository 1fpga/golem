use std::io::Cursor;
use std::path::PathBuf;

/// A ROM, including any information the core needs to know about the ROM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rom {
    /// A ROM that is stored in memory.
    Memory(Option<PathBuf>, Cursor<Vec<u8>>),

    /// A ROM that is stored in a file on the file system.
    File(PathBuf),
}
