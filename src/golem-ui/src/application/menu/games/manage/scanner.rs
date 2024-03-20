use sha2::Digest;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size: usize,
    pub sha1: sha1::digest::Output<sha1::Sha1Core>,
    pub sha256: sha2::digest::Output<sha2::Sha256>,
}

pub struct DirectoryScanner {
    nb_directories: AtomicU32,
    nb_files: AtomicU32,
    scanned: AtomicU32,
}

unsafe impl Sync for DirectoryScanner {}

impl DirectoryScanner {
    pub fn new() -> Self {
        Self {
            nb_directories: AtomicU32::new(0),
            nb_files: AtomicU32::new(0),
            scanned: AtomicU32::new(0),
        }
    }

    #[inline]
    pub fn total(&self) -> u32 {
        self.nb_directories.load(Ordering::Relaxed) + self.nb_files.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn scanned(&self) -> u32 {
        self.scanned.load(Ordering::Relaxed)
    }

    pub fn calc_dir(&self, dir: &Path) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                self.nb_directories.fetch_add(1, Ordering::Relaxed);
                self.calc_dir(&path);
                self.scanned.fetch_add(1, Ordering::Relaxed);
            } else {
                self.nb_files.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn scan_dir(&self, dir: &Path, queue: &crossbeam_queue::ArrayQueue<ScanResult>) {
        // Walk through all directories and read all files.
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            self.scanned.fetch_add(1, Ordering::Relaxed);

            let size = entry.metadata().unwrap().len() as usize;
            let content = std::fs::read(entry.path()).unwrap();
            let sha256 = sha2::Sha256::digest(&content);
            let sha1 = sha1::Sha1::digest(&content);
            let path = entry.path().to_path_buf();

            // Try to push and block if the queue is full.
            'queue_push: loop {
                if queue
                    .push(ScanResult {
                        path: path.clone(),
                        size,
                        sha1,
                        sha256,
                    })
                    .is_ok()
                {
                    break 'queue_push;
                }

                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}
