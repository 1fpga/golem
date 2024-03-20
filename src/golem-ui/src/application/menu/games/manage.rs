use crate::application::menu::cores::select_core;
use crate::application::menu::filesystem::{select_file_path_menu, FilesystemMenuOptions};
use crate::application::menu::games::manage::scanner::ScanResult;
use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, TextMenuOptions};
use crate::application::panels::alert::{alert, show_error};
use crate::application::panels::progress::{progress_bar, ProgressBarUpdate};
use crate::application::GoLEmApp;
use golem_db::models;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tracing::info;

mod identifier;
mod scanner;

fn scan_directory(app: &mut GoLEmApp) {
    let dir = select_file_path_menu(
        app,
        "Select Directory",
        std::env::current_exe().unwrap().parent().unwrap(),
        FilesystemMenuOptions::default()
            .with_allow_back(true)
            .with_select_dir(),
    )
    .unwrap();

    let dir = match dir {
        Some(dir) => dir,
        None => return,
    };

    info!(?dir, "Scanning directory");

    let done_scanning = AtomicBool::new(false);
    let done_identifying = AtomicBool::new(false);
    let queue = crossbeam_queue::ArrayQueue::new(1024);
    let p = scanner::DirectoryScanner::new();
    let db = app.database();
    let to_identify = AtomicUsize::new(0);
    let identified = AtomicUsize::new(0);

    crossbeam_utils::thread::scope(|s| {
        // One thread to scan all the files.
        let j1 = s.spawn(|_| {
            p.calc_dir(&dir);
            p.scan_dir(&dir, &queue);

            done_scanning.store(true, Ordering::Relaxed);
        });

        // One thread to send them all to retronomicon.
        let j2 = s.spawn(|_| {
            loop {
                let mut db = db.lock().unwrap();
                let d = identifier::GameIdentifier::from_db(&mut db).unwrap();

                const NONE: Option<ScanResult> = None;
                let mut buffer = [NONE; 512];
                let mut i = 0;
                loop {
                    if let Some(v) = queue.pop() {
                        buffer[i] = Some(v);
                        i += 1;
                        if i == 512 {
                            break;
                        }
                    } else if done_scanning.load(Ordering::Relaxed) {
                        break;
                    } else {
                        continue;
                    }
                }

                if i > 0 {
                    let mut v = Vec::new();
                    v.extend(buffer.iter_mut().take(i).map(|v| v.take().unwrap()));

                    to_identify.fetch_add(i, Ordering::Relaxed);
                    d.search_and_create(&mut db, v).unwrap();
                    identified.fetch_add(i, Ordering::Relaxed);
                }

                if done_scanning.load(Ordering::Relaxed) && queue.is_empty() {
                    break;
                }
            }

            done_identifying.store(true, Ordering::Relaxed);
        });

        progress_bar(app, "Scanning Directory", 0, || {
            let n = p.scanned() + identified.load(Ordering::Relaxed) as u32;
            let total = p.total() + to_identify.load(Ordering::Relaxed) as u32;

            if done_scanning.load(Ordering::Relaxed) && done_identifying.load(Ordering::Relaxed) {
                ProgressBarUpdate::Done
            } else {
                ProgressBarUpdate::UpdateBarTotal(n, total)
            }
        });

        j1.join().unwrap();
        j2.join().unwrap();
    })
    .unwrap();
}

fn add_dat_file_(app: &mut GoLEmApp) {
    let x = alert(
        app,
        "Adding DAT files",
        "This happens in two steps:\n\
        1. Select the DAT file you want to add from the filesystem.\n\
        2. Select the core to use to load games found in the DAT. A\n\
           core can have multiple DAT, but a DAT can only apply to a\n\
           single core.\
    ",
        &["Okay", "Cancel"],
    );

    if x != Some(0) {
        return;
    }

    let dat_path = select_file_path_menu(
        app,
        "Select DAT file",
        std::env::current_exe().unwrap().parent().unwrap(),
        FilesystemMenuOptions::default().with_allow_back(true),
    )
    .ok()
    .flatten();

    let dat_path = match dat_path {
        Some(dat_path) => dat_path,
        None => return,
    };

    let datfile = match datary::read_file(&dat_path) {
        Ok(datfile) => datfile,
        Err(e) => {
            show_error(app, e, true);
            return;
        }
    };

    let core = match select_core(app, "Select Core") {
        Some(core) => core,
        None => return,
    };

    let db = app.database();
    let path = dat_path.to_str().unwrap();
    let filename = dat_path.file_name().unwrap().to_str().unwrap();
    models::DatFile::create(
        db.lock().unwrap().deref_mut(),
        datfile
            .header
            .as_ref()
            .map(|h| h.name.as_str())
            .unwrap_or(filename),
        path,
        &core,
        0,
    )
    .unwrap();

    alert(app, "Success", "DAT file added successfully.", &["Okay"]);
}

#[derive(Copy, Clone, PartialEq)]
enum Menu {
    Back,
    ScanDirectory,
    AddDatFile,
}

impl MenuReturn for Menu {
    fn back() -> Option<Self> {
        Some(Menu::Back)
    }
}

pub fn manage_games(app: &mut GoLEmApp) {
    let mut state = None;

    loop {
        let (selection, new_state) = text_menu(
            app,
            "Manage Games",
            &[
                ("Scan Directory", "", Menu::ScanDirectory),
                ("Add DAT files to database", "", Menu::AddDatFile),
            ],
            TextMenuOptions::default().with_state(state),
        );

        state = Some(new_state);

        match selection {
            Menu::Back => break,
            Menu::ScanDirectory => {
                scan_directory(app);
            }
            Menu::AddDatFile => {
                add_dat_file_(app);
            }
        }
    }
}
