use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tracing::trace;

#[derive(Debug)]
enum SdMountFileInner {
    /// A memory based sd card.
    Memory(Cursor<Vec<u8>>),

    /// A file that is mounted to the core, and is potentially persisted
    /// on the filesystem.
    File {
        f: Option<File>,

        /// The path to the file on the filesystem. If no path is specified,
        /// this file is not persisted.
        path: Option<PathBuf>,
    },
}

impl SdMountFileInner {}

impl Read for SdMountFileInner {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            SdMountFileInner::Memory(data) => data.read(buf),
            SdMountFileInner::File { f: Some(f), .. } => f.read(buf),
            SdMountFileInner::File { .. } => Ok(0),
        }
    }
}

impl Write for SdMountFileInner {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            SdMountFileInner::Memory(data) => data.write(buf),
            SdMountFileInner::File { f: Some(f), .. } => f.write(buf),
            SdMountFileInner::File { path: Some(p), .. } => {
                trace!("Creating {:?}", p);
                let mut f = File::create(&p)?;
                let result = f.write(buf);
                *self = SdMountFileInner::File {
                    f: Some(f),
                    path: Some(p.clone()),
                };
                result
            }
            SdMountFileInner::File { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "File is not writable",
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            SdMountFileInner::Memory(data) => data.flush(),
            SdMountFileInner::File { f: Some(f), .. } => f.flush(),
            SdMountFileInner::File { .. } => Ok(()),
        }
    }
}

impl Seek for SdMountFileInner {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            SdMountFileInner::Memory(cursor) => cursor.seek(pos),
            SdMountFileInner::File { f: Some(f), .. } => f.seek(pos),
            SdMountFileInner::File { path: Some(p), .. } => {
                trace!("Creating {:?}", p);
                let mut f = File::create(&p)?;
                let result = f.seek(pos);
                *self = SdMountFileInner::File {
                    f: Some(f),
                    path: Some(p.clone()),
                };
                result
            }
            SdMountFileInner::File { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "File is not writable",
            )),
        }
    }
}

/// A file that can be mounted to the core. The core can write
/// to it (if writable on the filesystem), and it should be
/// persisted on the filesystem.
#[derive(Debug)]
pub struct SdCard {
    writeable: bool,
    can_grow: bool,

    inner: SdMountFileInner,
}

impl SdCard {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let mut writeable = true;
        let file = if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap())
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            None
        } else if let Ok(f) = OpenOptions::new().write(true).read(true).open(&path) {
            writeable = true;
            Some(f)
        } else if let Ok(f) = OpenOptions::new().read(true).open(&path) {
            writeable = false;
            Some(f)
        } else {
            None
        };

        Ok(Self {
            writeable,
            can_grow: true,
            inner: SdMountFileInner::File {
                f: file,
                path: Some(path),
            },
        })
    }

    pub fn writeable(&self) -> bool {
        self.writeable
    }

    pub fn size(&self) -> u64 {
        match &self.inner {
            SdMountFileInner::Memory(data) => data.get_ref().len() as u64,
            SdMountFileInner::File { f: Some(f), .. } => f.metadata().map(|m| m.len()).unwrap_or(0),
            SdMountFileInner::File { .. } => 0,
        }
    }

    pub fn as_io(&mut self) -> &'_ mut (impl Read + Write + Seek) {
        &mut self.inner
    }
}
