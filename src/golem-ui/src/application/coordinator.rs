use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use tracing::{info, trace};

use golem_db::models::Core as DbCore;
use golem_db::models::CoreFile as DbCoreFile;
use golem_db::models::Game as DbGame;
use golem_db::Connection;
use mister_fpga::core::file::SdCard;
use mister_fpga::core::MisterFpgaCore;
use one_fpga::core::SaveState;
use one_fpga::runner::CoreLaunchInfo;
use one_fpga::{Core, GolemCore};

use crate::application::GoLEmApp;
use crate::data::paths;

#[derive(Default, Debug, Clone, Copy)]
pub struct GameStartInfo {
    core_id: Option<i32>,
    game_id: Option<i32>,
    save_state_id: Option<i32>,
}

impl GameStartInfo {
    pub fn with_core_id(mut self, core_id: i32) -> Self {
        self.core_id = Some(core_id);
        self
    }

    pub fn with_maybe_core_id(mut self, core_id: Option<i32>) -> Self {
        self.core_id = core_id;
        self
    }

    pub fn with_game_id(mut self, game_id: i32) -> Self {
        self.game_id = Some(game_id);
        self
    }

    pub fn with_save_state_id(mut self, save_state_id: i32) -> Self {
        self.save_state_id = Some(save_state_id);
        self
    }
}

struct CoordinatorInner {
    current_core: Option<DbCore>,
    current_game: Option<DbGame>,
    current_sav: Option<DbCoreFile>,

    database: Arc<Mutex<Connection>>,
}

impl Debug for CoordinatorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorInner")
            .field("current_core", &self.current_core)
            .field("current_game", &self.current_game)
            .finish()
    }
}

impl CoordinatorInner {
    pub(super) fn new(database: Arc<Mutex<Connection>>) -> Self {
        Self {
            database,
            current_core: None,
            current_game: None,
            current_sav: None,
        }
    }

    pub fn launch_game(
        &mut self,
        app: &mut GoLEmApp,
        info: CoreLaunchInfo<GameStartInfo>,
    ) -> Result<(bool, GolemCore), String> {
        info!(?info, "Starting game");
        let mut database = self.database.lock().unwrap();

        // Load the core if necessary. If it's the same core, we don't need to.
        let (mut golem_core, core): (GolemCore, DbCore) = match info.data.core_id {
            Some(id) => {
                let db_core = DbCore::get(&mut database, id)
                    .map_err(|e| e.to_string())?
                    .ok_or("Core not found")?;
                (
                    app.platform_mut()
                        .core_manager_mut()
                        .load_core(&db_core.path)?,
                    db_core,
                )
            }
            None => {
                let db_core = self.current_core.clone().ok_or("No core selected")?;
                (
                    app.platform_mut()
                        .core_manager_mut()
                        .get_current_core()
                        .ok_or("No core running")?,
                    db_core,
                )
            }
        };

        let c = golem_core
            .as_any_mut()
            .downcast_mut::<MisterFpgaCore>()
            .ok_or("Core is not a MisterFpgaCore")?;

        self.current_core = Some(core);
        self.current_game = None;
        let mut should_show_menu = true;

        // Load the game
        if let Some(game_id) = info.data.game_id {
            let mut game = DbGame::get(&mut database, game_id)
                .map_err(|e| e.to_string())?
                .ok_or("Game not found")?;
            let game_path = game
                .path
                .as_ref()
                .ok_or("No file path for game")?
                .to_string();
            let core_file = golem_db::models::CoreFile::latest_for_game(&mut database, game_id)
                .map_err(|e| e.to_string())?;

            let should_sav = c
                .menu_options()
                .iter()
                .filter_map(|x| x.as_load_file_info())
                .any(|i| i.save_support);

            c.load_file(Path::new(&game_path), None)?;
            if should_sav {
                // Mount the SAV file.
                let game_name = game.name.clone();

                if let Some(core_file) = core_file {
                    c.mount(SdCard::from_path(Path::new(&core_file.path)).unwrap(), 0)?;
                } else {
                    info!("No SAV file found for game, creating one.");
                    let path = paths::sav_path(c.name()).join(format!("{}.sav", game_name));
                    c.mount(SdCard::from_path(Path::new(&path)).unwrap(), 0)
                        .map_err(|e| e.to_string())?;
                }
            }
            c.end_send_file()?;

            c.poll_mounts()?;

            // Record in the Database the time we launched this game, ignoring
            // errors.
            let _ = game.play(&mut database);
            self.current_game = Some(game);
            should_show_menu = false;
        }

        // Load all savestates for this game.
        if let Some(game) = &self.current_game {
            let db_ss = golem_db::models::SaveState::list_for_game(&mut database, game.id)
                .map_err(|e| e.to_string())?;

            for (slot, ss) in db_ss.iter().enumerate() {
                if let Ok(Some(core_ss)) = c.save_state_mut(slot) {
                    let mut f = std::fs::File::open(&ss.path).map_err(|e| e.to_string())?;
                    core_ss.load(&mut f).map_err(|e| e.to_string())?;
                }
            }
        }

        // Load the savestate requested in the first slot.
        if let Some(savestate) = info.data.save_state_id {
            let savestate = golem_db::models::SaveState::get(&mut database, savestate)
                .map_err(|e| e.to_string())?
                .ok_or("Savestate not found")?;

            if let Some(state) = c.save_states_mut().and_then(|x| x.slots_mut().get_mut(0)) {
                let mut f = std::fs::File::open(savestate.path).map_err(|e| e.to_string())?;
                state.load(&mut f).map_err(|e| e.to_string())?;
            }
        }

        Ok((should_show_menu, golem_core))
    }

    pub fn create_savestate(
        &mut self,
        slot: usize,
        screenshot: Option<&DynamicImage>,
    ) -> Result<std::fs::File, String> {
        let core = self.current_core.as_ref().ok_or("No core loaded")?;
        let game = self.current_game.as_ref().ok_or("No game loaded")?;
        let mut database = self.database.lock().unwrap();
        let ss_root = paths::savestates_path(&core.slug);
        if !ss_root.exists() {
            std::fs::create_dir_all(&ss_root).map_err(|e| e.to_string())?;
        }

        let name = &game.name;
        let savestate_path: PathBuf = ss_root.join(format!("{name}_{slot}")).with_extension("ss");
        let screenshot_path = if screenshot.is_some() {
            Some(savestate_path.with_extension("ss.png"))
        } else {
            None
        };

        trace!(?savestate_path, ?screenshot_path, "Saving save state");

        if let Some(s) = screenshot {
            s.save(screenshot_path.clone().unwrap())
                .map_err(|e| e.to_string())?;
        }

        let ss = golem_db::models::SaveState::create(
            &mut database,
            core.id,
            game.id,
            savestate_path.to_string_lossy().to_string(),
            screenshot_path.map(|x| x.to_string_lossy().to_string()),
        )
        .map_err(|e| e.to_string())?;

        let f = std::fs::File::create(ss.path).map_err(|e| e.to_string())?;
        Ok(f)
    }
}

#[derive(Debug, Clone)]
pub struct Coordinator {
    inner: Arc<Mutex<CoordinatorInner>>,
}

impl Coordinator {
    pub fn new(database: Arc<Mutex<Connection>>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CoordinatorInner::new(database))),
        }
    }

    pub fn launch_game(
        &mut self,
        app: &mut GoLEmApp,
        info: CoreLaunchInfo<GameStartInfo>,
    ) -> Result<(bool, GolemCore), String> {
        self.inner.lock().unwrap().launch_game(app, info)
    }

    pub fn current_core(&self) -> Option<DbCore> {
        self.inner.lock().unwrap().current_core.clone()
    }

    pub fn current_game(&self) -> Option<DbGame> {
        self.inner.lock().unwrap().current_game.clone()
    }

    pub fn current_sav(&self) -> Option<DbCoreFile> {
        self.inner.lock().unwrap().current_sav.clone()
    }

    pub fn create_savestate(
        &self,
        slot: usize,
        screenshot: Option<&DynamicImage>,
    ) -> Result<std::fs::File, String> {
        self.inner
            .lock()
            .unwrap()
            .create_savestate(slot, screenshot)
    }
}
