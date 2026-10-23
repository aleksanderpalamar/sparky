use std::path::{Path, PathBuf};

use sparky_theme::locations;

pub const ZSH: &str = "zsh";
const RC: &str = ".zshrc";
const FOLDER: &str = "zsh";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dirs {
    pub user: PathBuf,
    pub system: PathBuf,
}

pub fn dirs(config_home: Option<&str>, home: &str, sparky_path: Option<&str>) -> Dirs {
    Dirs {
        user: locations::config_dir(config_home, home).join(FOLDER),
        system: locations::sparky_path(sparky_path).join(FOLDER),
    }
}

pub fn chosen(dirs: &Dirs, exists: impl Fn(&Path) -> bool) -> PathBuf {
    match exists(&dirs.user.join(RC)) {
        true => dirs.user.clone(),
        false => dirs.system.clone(),
    }
}
