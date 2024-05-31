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
        /// The file handle. If this is `None`, the file has not been created
        /// yet.
        f: Option<File>,

        /// The path to the file on the filesystem. If no path is specified,
        /// this file is not persisted.
        path: Option<PathBuf>,

        /// The maximum size of the file. If the file grows beyond this size,
        /// it will be truncated. If this is `None` the file can grow as large
        /// as the filesystem allows.
        max_size: Option<u64>,
    },
}

impl one_fpga::core::MountedFile for SdMountFileInner {}

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
            SdMountFileInner::File {
                f: Some(f),
                max_size: None,
                ..
            } => f.write(buf),
            SdMountFileInner::File {
                f: Some(f),
                max_size: Some(max_size),
                ..
            } => {
                if f.stream_position()? + buf.len() as u64 > *max_size {
                    let end = (*max_size).saturating_sub(f.stream_position()?) as usize;
                    f.write(&buf[..end])
                } else {
                    f.write(buf)
                }
            }
            SdMountFileInner::File {
                path: Some(p),
                max_size,
                ..
            } => {
                if let Some(m) = max_size {
                    if buf.len() > *m as usize {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Buffer is larger than max size",
                        ));
                    }
                }

                trace!("Creating {:?}", p);
                let mut f = File::create(&p)?;
                let result = f.write(buf);
                *self = SdMountFileInner::File {
                    f: Some(f),
                    path: Some(p.clone()),
                    max_size: *max_size,
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
            SdMountFileInner::File {
                path: Some(p),
                max_size,
                ..
            } => {
                trace!("Creating {:?}", p);
                let mut f = File::create(&p)?;
                let result = f.seek(pos);
                *self = SdMountFileInner::File {
                    f: Some(f),
                    path: Some(p.clone()),
                    max_size: *max_size,
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
            inner: SdMountFileInner::File {
                f: file,
                path: Some(path),
                max_size: None,
            },
        })
    }

    pub fn from_memory(data: Vec<u8>) -> Self {
        Self {
            writeable: true,
            inner: SdMountFileInner::Memory(Cursor::new(data)),
        }
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

    pub fn len(&self) -> usize {
        self.size() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn as_io(&mut self) -> &'_ mut (impl Read + Write + Seek) {
        &mut self.inner
    }

    pub fn as_mounted(&mut self) -> &'_ mut dyn one_fpga::core::MountedFile {
        &mut self.inner
    }
}
