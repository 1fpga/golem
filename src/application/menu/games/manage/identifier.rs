use crate::application::menu::games::manage::scanner::ScanResult;
use datary::optimize::OptimizedDatafile;
use datary::Datafile;
use golem_db::models::{Core, DatFile, Game};
use golem_db::Connection;
use std::path::PathBuf;
use tracing::info;

fn create_games_from_datfile_(
    db: &mut Connection,
    datafile: &OptimizedDatafile,
    core: &Core,
    shas: Vec<ScanResult>,
) -> Result<Vec<ScanResult>, anyhow::Error> {
    info!("Identifying {} files...", shas.len());
    let remaining = shas
        .into_iter()
        .filter_map(|scan_result| {
            let sha = hex::encode(scan_result.sha1.as_slice());
            let games = datafile.games_by_sha1(&sha);
            for g in games.unwrap_or_default() {
                if g.roms.iter().any(|r| r.size == scan_result.size) {
                    if let Err(e) = Game::create(
                        db,
                        g.name.clone(),
                        core,
                        &scan_result.path,
                        g.description.clone(),
                    ) {
                        return Some(Err(e));
                    }
                    return None;
                }
            }
            Some(Ok(scan_result))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(remaining)
}

/// A game identifier, using either Retronomicon or a DAT file.
pub enum GameIdentifier {
    DatFile(OptimizedDatafile, Box<Core>),
    Combined(Vec<GameIdentifier>),
}

impl GameIdentifier {
    pub fn from_dat_file(datafile: Datafile, core: Core) -> Self {
        Self::DatFile(datafile.into(), Box::new(core))
    }

    pub fn from_db(db: &mut Connection) -> Result<Self, anyhow::Error> {
        let mut identifiers = Vec::new();
        for d in DatFile::list(db)? {
            let p = PathBuf::from(d.path);
            if p.exists() && !p.is_dir() {
                let datafile = datary::read_file(p)?;
                if let Some(core) = Core::get(db, d.core_id)? {
                    identifiers.push(Self::from_dat_file(datafile, core));
                }
            }
        }
        Ok(Self::Combined(identifiers))
    }

    pub fn search_and_create(
        &self,
        db: &mut Connection,
        scan: Vec<ScanResult>,
    ) -> Result<Vec<ScanResult>, anyhow::Error> {
        match self {
            Self::DatFile(datafile, core) => create_games_from_datfile_(db, datafile, core, scan),
            GameIdentifier::Combined(inner) => {
                let mut remaining = scan;
                for other in inner {
                    remaining = other.search_and_create(db, remaining)?;
                }
                Ok(remaining)
            }
        }
    }
}
