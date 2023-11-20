use crate::application::menu::games::manage::scanner::ScanResult;
use datary::optimize::OptimizedDatafile;
use datary::Datafile;
use golem_db::models::{Core, DatFile, Game};
use golem_db::{models, Connection};
use reqwest::Url;
use retronomicon_dto::routes;
use std::path::PathBuf;
use tracing::{info, warn};

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

pub fn create_games_from_retronomicon_(
    db: &mut Connection,
    url: &Url,
    scans: Vec<ScanResult>,
) -> Result<Vec<ScanResult>, anyhow::Error> {
    info!("Identifying {} files...", scans.len());

    let client = reqwest::blocking::Client::new();
    let url = routes::v1::games(&url);
    let response = client
        .get(url)
        .json(&retronomicon_dto::games::GameListBody {
            sha1: Some(scans.iter().map(|s| s.sha1.to_vec().into()).collect()),
            md5: None,
            sha256: None,
        })
        .send()?;
    let games: Vec<retronomicon_dto::games::GameListItemResponse> = response.json()?;

    let system_to_core = models::Core::list(db, 0, 1000, models::CoreOrder::NameAsc)
        .map(|c| {
            c.into_iter()
                .map(|c| (c.system_slug.clone(), c))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .collect::<std::collections::HashMap<_, _>>();
    let mut sha_to_scan = scans
        .into_iter()
        .map(|s| (s.sha1.to_vec(), s))
        .collect::<std::collections::HashMap<_, _>>();

    for g in games {
        let maybe_scan_found = g
            .artifacts
            .iter()
            .filter(|a| a.sha1.is_some())
            .find_map(|a| {
                let sha = a.sha1.as_ref().map(|s| s.to_vec()).unwrap();
                sha_to_scan.get(&sha)
            });
        let scan = if let Some(s) = maybe_scan_found {
            s
        } else {
            warn!(
                "Could not find artifact for game '{}', even though we asked for its sha",
                g.name
            );
            continue;
        };

        let maybe_core = system_to_core.get(&g.system_id.slug);
        let core = if let Some(core) = maybe_core {
            core
        } else {
            warn!("Could not find core for system {}", g.system_id.slug);
            continue;
        };

        Game::create(
            db,
            g.name.clone(),
            core,
            &scan.path,
            g.short_description.clone(),
        )?;

        sha_to_scan.remove(&scan.sha1.to_vec());
    }

    Ok(sha_to_scan.into_values().collect::<Vec<_>>())
}

/// A game identifier, using either Retronomicon or a DAT file.
pub enum GameIdentifier {
    DatFile(OptimizedDatafile, Box<Core>),
    Retronomicon(Url),
    Combined(Vec<GameIdentifier>),
}

impl GameIdentifier {
    pub fn from_dat_file(datafile: Datafile, core: Core) -> Self {
        Self::DatFile(datafile.into(), Box::new(core))
    }

    pub fn from_db(db: &mut Connection) -> Result<Self, anyhow::Error> {
        let mut identifiers = Vec::new();
        identifiers.push(Self::Retronomicon(
            Url::parse("https://retronomicon.land/").unwrap(),
        ));
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
            Self::Retronomicon(url) => create_games_from_retronomicon_(db, url, scan),
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
