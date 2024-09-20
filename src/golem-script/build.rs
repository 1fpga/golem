use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("migrations");
    let in_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../golem-frontend/migrations");
    let in_glob = in_dir.join("**/*.sql");

    for file in glob::glob(in_glob.to_string_lossy().as_ref()).unwrap() {
        let file = file.unwrap();
        println!("cargo::rerun-if-changed={}", file.display());
        println!("cargo::rerun-if-changed={}", in_dir.display());
        let rel_path = file.strip_prefix(&in_dir).unwrap();
        let out_path = out_dir.join(rel_path);

        let sql = fs::read_to_string(&file).unwrap();
        let mut sql_out = String::new();
        for line in sql.split("\n") {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let line = if let Some(pos) = line.find("--") {
                line.get(..pos).unwrap()
            } else {
                line
            };

            sql_out.push_str(line.replace("  ", " ").as_str());
        }

        fs::create_dir_all(out_path.parent().unwrap()).unwrap();
        fs::write(out_path, sql_out).unwrap();
    }
}
