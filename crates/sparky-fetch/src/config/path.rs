use std::path::{Path, PathBuf};

const FOLDER: &str = "sparky-fetch";
const FILE: &str = "config.toml";

pub fn config_path(config_home: Option<&str>, home: &str) -> PathBuf {
    let base = match config_home.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => PathBuf::from(path),
        None => Path::new(home).join(".config"),
    };
    base.join(FOLDER).join(FILE)
}
