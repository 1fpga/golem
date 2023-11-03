#![cfg(feature = "optimized")]
use crate::dat::{Datafile, Game};
use ouroboros::self_referencing;
use std::collections::{BTreeMap, HashMap};

fn build_sha1_cache_(datafile: &Datafile) -> HashMap<&str, Vec<&Game>> {
    let mut map = HashMap::new();
    for g in &datafile.games {
        for r in &g.roms {
            if let Some(sha1) = &r.sha1 {
                map.entry(sha1.as_str()).or_insert_with(Vec::new).push(g);
            }
        }
    }
    map
}

fn build_crc_cache_(datafile: &Datafile) -> HashMap<&str, Vec<&Game>> {
    let mut map = HashMap::new();
    for g in &datafile.games {
        for r in &g.roms {
            if let Some(crc) = &r.crc {
                map.entry(crc.as_str()).or_insert_with(Vec::new).push(g);
            }
        }
    }
    map
}

fn build_md5_cache_(datafile: &Datafile) -> HashMap<&str, Vec<&Game>> {
    let mut map = HashMap::new();
    for g in &datafile.games {
        for r in &g.roms {
            if let Some(md5) = &r.md5 {
                map.entry(md5.as_str()).or_insert_with(Vec::new).push(g);
            }
        }
    }
    map
}

fn build_size_cache_(datafile: &Datafile) -> HashMap<usize, Vec<&Game>> {
    let mut map = HashMap::new();
    for g in &datafile.games {
        for r in &g.roms {
            map.entry(r.size).or_insert_with(Vec::new).push(g);
        }
    }
    map
}

fn build_rom_name_cache_(datafile: &Datafile) -> BTreeMap<&str, &Game> {
    let mut map = BTreeMap::new();
    for g in &datafile.games {
        for r in &g.roms {
            debug_assert!(map.get(r.name.as_str()).is_none());
            map.insert(r.name.as_str(), g);
        }
    }
    map
}

/// An optimized version of the Datfile with various caches to search for games.
#[self_referencing]
pub struct OptimizedDatafile {
    /// The original datafile.
    datafile: Datafile,

    /// A map of sha1 to games.
    #[borrows(datafile)]
    #[covariant]
    sha1: HashMap<&'this str, Vec<&'this Game>>,

    /// A map of crc to games.
    #[borrows(datafile)]
    #[covariant]
    crc: HashMap<&'this str, Vec<&'this Game>>,

    /// A map of md5 to games.
    #[borrows(datafile)]
    #[covariant]
    md5: HashMap<&'this str, Vec<&'this Game>>,

    /// A map of sizes to games.
    #[borrows(datafile)]
    #[covariant]
    size: HashMap<usize, Vec<&'this Game>>,

    /// A map of ROM names to games. Can do range searching.
    #[borrows(datafile)]
    #[covariant]
    rom_names: BTreeMap<&'this str, &'this Game>,
}

impl From<Datafile> for OptimizedDatafile {
    fn from(datafile: Datafile) -> Self {
        OptimizedDatafileBuilder {
            datafile,
            sha1_builder: build_sha1_cache_,
            crc_builder: build_crc_cache_,
            md5_builder: build_md5_cache_,
            size_builder: build_size_cache_,
            rom_names_builder: build_rom_name_cache_,
        }
        .build()
    }
}

impl OptimizedDatafile {
    pub fn game_by_sha1(&self, sha1: &str) -> Option<&Game> {
        self.borrow_sha1().get(sha1).map(|v| v[0])
    }

    pub fn games_by_sha1(&self, sha1: &str) -> Option<&[&Game]> {
        self.borrow_sha1().get(sha1).map(|v| &v[..])
    }

    pub fn game_by_crc(&self, crc: &str) -> Option<&Game> {
        self.borrow_crc().get(crc).map(|v| v[0])
    }

    pub fn games_by_crc(&self, crc: &str) -> Option<&[&Game]> {
        self.borrow_crc().get(crc).map(|v| &v[..])
    }

    pub fn game_by_md5(&self, md5: &str) -> Option<&Game> {
        self.borrow_md5().get(md5).map(|v| v[0])
    }

    pub fn games_by_md5(&self, md5: &str) -> Option<&[&Game]> {
        self.borrow_md5().get(md5).map(|v| &v[..])
    }

    pub fn games_by_size(&self, size: usize) -> Option<&[&Game]> {
        self.borrow_size().get(&size).map(|v| &v[..])
    }

    pub fn game_by_name(&self, name: &str) -> Option<&Game> {
        self.borrow_rom_names().get(name).copied()
    }

    pub fn games_by_name_prefix<'this, 'a: 'this>(
        &'this self,
        name: &'a str,
    ) -> impl Iterator<Item = &'this Game> {
        self.borrow_rom_names()
            .range(name..)
            .take_while(move |(k, _)| k.starts_with(name))
            .map(|(_, v)| *v)
    }
}
