use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

use image::DynamicImage;

use golem_db::Connection;

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

struct CoordinatorInner {}

impl Debug for CoordinatorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorInner").finish()
    }
}

impl CoordinatorInner {
    pub(super) fn new(_database: Arc<Mutex<Connection>>) -> Self {
        Self {}
    }

    pub fn create_savestate(
        &mut self,
        _slot: usize,
        _screenshot: Option<&DynamicImage>,
    ) -> Result<std::fs::File, String> {
        // let core = self.current_core.as_ref().ok_or("No core loaded")?;
        // let game = self.current_game.as_ref().ok_or("No game loaded")?;
        // let mut database = self.database.lock().unwrap();
        // let ss_root = paths::savestates_path(&core.slug);
        // if !ss_root.exists() {
        //     std::fs::create_dir_all(&ss_root).map_err(|e| e.to_string())?;
        // }
        //
        // let name = &game.name;
        // let savestate_path: PathBuf = ss_root.join(format!("{name}_{slot}")).with_extension("ss");
        // let screenshot_path = if screenshot.is_some() {
        //     Some(savestate_path.with_extension("ss.png"))
        // } else {
        //     None
        // };
        //
        // trace!(?savestate_path, ?screenshot_path, "Saving save state");
        //
        // if let Some(s) = screenshot {
        //     s.save(screenshot_path.clone().unwrap())
        //         .map_err(|e| e.to_string())?;
        // }
        //
        // let ss = golem_db::models::SaveState::create(
        //     &mut database,
        //     core.id,
        //     game.id,
        //     savestate_path.to_string_lossy().to_string(),
        //     screenshot_path.map(|x| x.to_string_lossy().to_string()),
        // )
        // .map_err(|e| e.to_string())?;
        //
        // let f = std::fs::File::create(ss.path).map_err(|e| e.to_string())?;
        // Ok(f)
        unreachable!("Not implemented anymore.")
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

    pub fn create_savestate(
        &self,
        _slot: usize,
        _screenshot: Option<&DynamicImage>,
    ) -> Result<std::fs::File, String> {
        unreachable!("Not implemented anymore.")
    }
}
