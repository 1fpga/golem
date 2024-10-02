use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatafileRoot {
    datafile: Datafile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Datafile {
    #[serde(alias = "game", default)]
    pub games: Vec<Game>,
}

impl Datafile {
    /// For each games, remove the sources that have the same size, crc32, md5, sha1 and sha256.
    pub fn minimize(&mut self) {
        for game in &mut self.games {
            game.metadata = game.archive.take();
            if let Some(metadata) = &mut game.metadata {
                metadata.languages.as_mut().map(Languages::normalize);
            }

            let mut all_sources = BTreeMap::new();
            game.sources = game
                .sources
                .iter()
                .filter(|source| {
                    if source.files.is_empty() {
                        return false;
                    }
                    let key = (
                        source.files[0].size,
                        &source.files[0].crc32,
                        &source.files[0].md5,
                        &source.files[0].sha1,
                        &source.files[0].sha256,
                    );
                    if let std::collections::btree_map::Entry::Vacant(e) = all_sources.entry(key) {
                        e.insert(());
                        true
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Game {
    #[serde(alias = "@name")]
    pub name: String,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Archive>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive: Option<Archive>,

    #[serde(alias = "source", default)]
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Languages {
    Array(Vec<String>),
    Single(String),
    None()
}

impl Languages {
    pub fn normalize(&mut self) {
        match self {
            Languages::Array(array) => {
                if array.len() == 1 {
                    *self = Languages::Single(array[0].clone());
                } else if array.is_empty() {
                    *self = Languages::None();
                }
            }
            Languages::Single(s) => {
                if s.is_empty() {
                    *self = Languages::None();
                } else if s.contains(',') {
                    *self = Languages::Array(s.split(',').map(str::to_string).collect());
                }
            }
            Languages::None() => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Archive {
    #[serde(alias = "@number")]
    pub number: String,

    #[serde(alias = "@name")]
    pub name: String,

    #[serde(alias = "@name_alt", skip_serializing_if = "Option::is_none")]
    pub name_alt: Option<String>,

    #[serde(alias = "@region")]
    region: String,

    #[serde(alias = "@languages", skip_serializing_if = "Option::is_none")]
    languages: Option<Languages>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Source {
    #[serde(alias = "file", default)]
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct File {
    #[serde(alias = "@id")]
    pub id: u32,

    #[serde(alias = "@extension")]
    pub extension: String,

    #[serde(alias = "@size")]
    pub size: u32,

    #[serde(alias = "@crc32")]
    pub crc32: String,

    #[serde(alias = "@md5")]
    pub md5: String,

    #[serde(alias = "@sha1")]
    pub sha1: String,

    #[serde(alias = "@sha256")]
    pub sha256: String,

    #[serde(alias = "@header", skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

#[derive(clap::Parser)]
struct Args {
    input: PathBuf,

    #[clap(short, long)]
    output: Option<String>,
}

fn main() {
    let opts: Args = Args::parse();

    let file = std::fs::File::open(opts.input).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut datafile: Datafile = quick_xml::de::from_reader(reader).unwrap();

    datafile.minimize();

    let writer: Box<dyn Write> = if let Some(output) = opts.output {
        let out = std::fs::File::create(output).unwrap();
        Box::new(std::io::BufWriter::new(out))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };

    serde_json::to_writer(writer, &datafile).unwrap()
}
