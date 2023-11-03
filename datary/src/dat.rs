//! The DAT file format and types.
//!
//! This has been taken from the DTD at http://www.logiqx.com/Dats/datafile.dtd
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForceMerge {
    Full,
    #[default]
    Split,
    None,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForceNoDump {
    #[default]
    Obsolete,
    Required,
    Ignore,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForcePack {
    #[default]
    Zip,
    Unzip,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RomMode {
    Merged,
    #[default]
    Split,
    Unmerged,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BiosMode {
    Merged,
    #[default]
    Split,
    Unmerged,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SampleMode {
    #[default]
    Merged,
    Unmerged,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockRomMode {
    Yes,
    #[default]
    No,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockBiosMode {
    Yes,
    #[default]
    No,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockSampleMode {
    Yes,
    #[default]
    No,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IsBios {
    Yes,
    #[default]
    No,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Default {
    Yes,
    #[default]
    No,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    BadDump,
    NoDump,
    #[default]
    Good,
    Verified,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Datafile {
    #[serde(rename = "@build")]
    pub build: Option<String>,

    #[serde(rename = "@debug", default)]
    pub debug: bool,

    pub header: Option<Header>,

    #[serde(rename = "game", default)]
    pub games: Vec<Game>,
}

impl Datafile {
    pub fn parse<R: std::io::Read>(mut buffer: BufReader<R>) -> Result<Self, Error> {
        let dat: Datafile = quick_xml::de::from_reader(&mut buffer)?;
        Ok(dat)
    }

    #[cfg(feature = "optimized")]
    pub fn optimize(self) -> crate::optimize::OptimizedDatafile {
        self.into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub version: String,
    pub author: String,
    pub email: Option<String>,
    pub homepage: Option<String>,
    pub url: Option<String>,
    pub comment: Option<String>,

    #[serde(rename = "clrmamepro")]
    pub clr_mame_pro: Option<ClrMamePro>,

    #[serde(rename = "romcenter")]
    pub rom_center: Option<RomCenter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClrMamePro {
    #[serde(rename = "@header")]
    pub header: Option<String>,

    #[serde(rename = "@forcemerging", default)]
    pub force_merging: ForceMerge,

    #[serde(rename = "@forcenodump", default)]
    pub force_no_dump: ForceNoDump,

    #[serde(rename = "@forcepacking", default)]
    pub force_packing: ForcePack,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RomCenter {
    #[serde(rename = "@plugin")]
    pub plugin: String,

    #[serde(rename = "@rommode", default)]
    pub rom_mode: RomMode,

    #[serde(rename = "@biosmode", default)]
    pub bios_mode: BiosMode,

    #[serde(rename = "@samplemode", default)]
    pub sample_mode: SampleMode,

    #[serde(rename = "@lockrommode", default)]
    pub lock_rom_mode: LockRomMode,

    #[serde(rename = "@lockbiosmode", default)]
    pub lock_bios_mode: LockBiosMode,

    #[serde(rename = "@locksamplemode", default)]
    pub lock_sample_mode: LockSampleMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "@name")]
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub comment: Vec<String>,

    #[serde(rename = "@isbios", default)]
    pub is_bios: IsBios,

    #[serde(rename = "@cloneof")]
    pub clone_of: Option<String>,

    #[serde(rename = "@romof")]
    pub rom_of: Option<String>,

    #[serde(rename = "@sampleof")]
    pub sample_of: Option<String>,

    #[serde(rename = "@board")]
    pub board: Option<String>,

    #[serde(rename = "@rebuildto")]
    pub rebuild_to: Option<String>,

    /// The year of manufacture. Technically a PCDATA but should probably be treated
    /// as an integer.
    pub year: Option<String>,
    pub manufacturer: Option<String>,

    #[serde(rename = "release", default)]
    pub releases: Vec<Release>,

    #[serde(rename = "biosset", default)]
    pub bios_sets: Vec<BiosSet>,

    #[serde(rename = "rom", default)]
    pub roms: Vec<Rom>,

    #[serde(rename = "disk", default)]
    pub disks: Vec<Disk>,

    #[serde(rename = "sample", default)]
    pub samples: Vec<Sample>,

    #[serde(rename = "archive", default)]
    pub archives: Vec<Archive>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@region")]
    region: String,

    #[serde(rename = "@language")]
    language: Option<String>,

    #[serde(rename = "@date")]
    date: Option<String>,

    #[serde(rename = "@default", default)]
    default: Option<Default>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiosSet {
    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@description")]
    description: String,

    #[serde(rename = "@default", default)]
    default: Option<Default>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rom {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@size")]
    pub size: usize,
    #[serde(rename = "@crc")]
    pub crc: Option<String>,
    #[serde(rename = "@sha1")]
    pub sha1: Option<String>,
    #[serde(rename = "@md5")]
    pub md5: Option<String>,
    #[serde(rename = "@merge")]
    pub merge: Option<String>,
    #[serde(rename = "@status")]
    pub status: Option<Status>,
    #[serde(rename = "@date")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disk {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@sha1")]
    pub sha1: Option<String>,
    #[serde(rename = "@md5")]
    pub md5: Option<String>,
    #[serde(rename = "@merge")]
    pub merge: Option<String>,
    #[serde(rename = "@status")]
    pub status: Option<Status>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Archive {
    #[serde(rename = "@name")]
    pub name: String,
}
