use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatafileRoot {
    datafile: Datafile,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Datafile {
    #[serde(alias = "game", default)]
    pub games: Vec<Game>,
}

impl Datafile {
    /// For each games, remove the sources that have the same size, crc32, md5, sha1 and sha256.
    pub fn minimize(&mut self, remove_dupes: bool) {
        let mut all_files = BTreeSet::new();

        for game in &mut self.games {
            game.metadata = game.archive.take();
            if let Some(metadata) = &mut game.metadata {
                game.shortname = Some(metadata.name.clone());
                metadata.name = game.name.clone();
                metadata.languages.as_mut().map(Languages::normalize);
            }

            // Merge all sources' files into 1.
            let mut game_files = BTreeMap::new();
            for source in &game.sources {
                for f in &source.files {
                    let key = (f.size, f.extension.clone(), f.sha256.clone());
                    if remove_dupes && all_files.contains(&key) {
                        continue;
                    }

                    all_files.insert(key.clone());
                    game_files.insert(key, f.clone());
                }
            }

            if game_files.is_empty() {
                eprintln!("No files for game: {:?}", game);
                game.sources = vec![];
            } else {
                game.sources = vec![Source {
                    files: game_files.into_values().collect(),
                }];
            }
        }
    }

    /// Validate any invariants.
    pub fn validate(&self) -> Result<(), String> {
        let mut errored = false;
        let mut hashes = HashMap::new();
        for game in &self.games {
            for source in &game.sources {
                for file in &source.files {
                    let hash = (&file.size, &file.sha256, &file.extension);
                    if let Some(other) = hashes.get(&hash) {
                        eprintln!("Duplicate games: {:?} {:?}", game.name, other);
                        eprintln!("      with hash: {:?}", hash);
                        errored = true;
                    } else {
                        hashes.insert(hash, &game.name);
                    }
                }
            }
        }

        if errored {
            Err("Found validation errors, see above".to_string())
        } else {
            Ok(())
        }
    }

    pub fn merge(&mut self, other: Datafile) {
        let mut all_hashes = HashMap::new();

        for g in &self.games {
            for s in &g.sources {
                for f in &s.files {
                    all_hashes.insert(
                        (f.extension.clone(), f.size, f.sha256.clone()),
                        g.name.clone(),
                    );
                }
            }
        }

        let mut new_sources: HashMap<String, Source> = HashMap::new();

        for g in other.games {
            for s in &g.sources {
                for f in &s.files {
                    if let Some(existing) =
                        all_hashes.get(&(f.extension.clone(), f.size, f.sha256.clone()))
                    {
                        if existing == &g.name {
                            continue;
                        }

                        eprintln!("Duplicate game: \n    {:?}\n    {:?}", existing, g.name);
                    } else {
                        self.games.push(g.clone());
                        all_hashes.insert(
                            (f.extension.clone(), f.size, f.sha256.clone()),
                            g.name.clone(),
                        );
                    }
                }
            }
        }

        for g in &mut self.games {
            if let Some(s) = new_sources.remove(&g.name) {
                g.sources.push(s);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Game {
    #[serde(alias = "@name")]
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortname: Option<String>,

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
    None(),
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

    #[serde(alias = "@sha1")]
    pub sha1: String,

    #[serde(alias = "@sha256")]
    pub sha256: String,

    #[serde(alias = "@header", skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

#[derive(Clone, Default, strum::EnumString, strum::EnumIter, strum::Display)]
enum InputType {
    #[default]
    /// Database XML (from no-intro.org).
    #[strum(ascii_case_insensitive)]
    DbXml,

    /// Hardware Target Game Database, from
    /// https://github.com/frederic-mahe/Hardware-Target-Game-Database.
    #[strum(ascii_case_insensitive)]
    Htgdb,
}

#[derive(clap::Parser)]
struct Args {
    inputs: Vec<PathBuf>,

    #[clap(short, long)]
    output: Option<String>,

    /// Allow duplicated hashes (duplicates will be removed, only the first instance will
    /// be kept).
    #[clap(long)]
    allow_dupes: bool,
}

fn convert_htgb<R: BufRead>(reader: R) -> Datafile {
    let mut csv = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(reader);

    let mut datafile = Datafile::default();

    let mut games = HashMap::new();

    for result in csv.records() {
        let record = result.expect("Could not read CSV record");

        let (sha256, path, sha1, _md5, _crc32, size) = match record.len() {
            6 => (
                record.get(0).unwrap().to_string(),
                record.get(1).unwrap().to_string(),
                record.get(2).unwrap().to_string(),
                record.get(3).unwrap().to_string(),
                record.get(4).unwrap().to_string(),
                record.get(5).unwrap().parse::<u32>().unwrap_or(0),
            ),
            l => panic!("Invalid number of columns: {}", l),
        };

        let file_name = PathBuf::from(path);
        let game_name = file_name
            .file_stem()
            .expect("Could not get file stem")
            .to_string_lossy()
            .to_string();
        let ext = file_name
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let g = games.entry(game_name.clone()).or_insert_with(|| Game {
            name: game_name.clone(),
            shortname: None,
            metadata: None,
            archive: None,
            sources: vec![],
        });
        g.sources.push(Source {
            files: vec![File {
                id: 0,
                extension: ext,
                size,
                sha1,
                sha256,
                header: None,
            }],
        });
    }
    datafile.games = games.into_values().collect();

    datafile
}

fn main() {
    let opts: Args = Args::parse();
    let mut master_datafile = Datafile::default();

    for path in opts.inputs {
        let file = std::fs::File::open(&path).unwrap();
        let reader = std::io::BufReader::new(file);

        let datafile = match path
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .as_deref()
        {
            Some("xml") => quick_xml::de::from_reader(reader).expect("Could not read XML file"),
            Some("txt") | Some("csv") => convert_htgb(reader),
            _ => panic!("Unknown file type"),
        };

        master_datafile.merge(datafile);
    }

    master_datafile.minimize(opts.allow_dupes);
    master_datafile.validate().unwrap();

    let writer: Box<dyn Write> = if let Some(output) = opts.output {
        let out = std::fs::File::create(output).unwrap();
        Box::new(std::io::BufWriter::new(out))
    } else {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    };

    serde_json::to_writer(writer, &master_datafile).unwrap()
}
