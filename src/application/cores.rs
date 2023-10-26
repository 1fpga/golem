use mister_db::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub struct CoreManager {
    database: Arc<Mutex<Connection>>,
}

impl CoreManager {
    pub fn new(database: Arc<Mutex<Connection>>) -> Self {
        Self { database }
    }

    pub fn scan(&mut self, folder: impl AsRef<Path>) {
        let cores = WalkDir::new(folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some())
            .filter(|e| e.path().extension().unwrap() == "rbf");

        eprintln!("{:?}", cores.size_hint());

        // let cores = mister_db::core::find_all(&self.database.read().unwrap());
        // println!("{:?}", cores);
    }
}
