use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::adapters::{group_file, placement_file};
use crate::domain::groups::Groups;
use crate::domain::library::Library;
use crate::domain::placements::Placements;
use crate::ports::LibraryStore;

const GROUPS: &str = "sparky/launcher-groups";
const CATEGORIES: &str = "sparky/launcher-categories";

pub struct LibraryFiles {
    groups: PathBuf,
    categories: PathBuf,
}

impl LibraryFiles {
    pub fn new(config: &Path) -> Self {
        Self {
            groups: config.join(GROUPS),
            categories: config.join(CATEGORIES),
        }
    }

    pub fn from_env() -> Self {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_default();
        Self::new(&config)
    }
}

impl LibraryStore for LibraryFiles {
    fn load(&self) -> Result<Library, String> {
        Ok(Library {
            groups: Groups::new(group_file::read(&contents(&self.groups)?)),
            placements: Placements::new(placement_file::read(&contents(&self.categories)?)),
        })
    }

    fn save(&self, library: &Library) -> Result<(), String> {
        replace(&self.groups, &group_file::write(library.groups.list()))?;
        replace(
            &self.categories,
            &placement_file::write(library.placements.list()),
        )
    }
}

fn contents(file: &Path) -> Result<String, String> {
    match fs::read_to_string(file) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.to_string()),
    }
}

fn replace(file: &Path, text: &str) -> Result<(), String> {
    let parent = file.parent().ok_or("invalid launcher directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut staged = file.as_os_str().to_owned();
    staged.push(".new");
    fs::write(&staged, text).map_err(|error| error.to_string())?;
    fs::rename(&staged, file).map_err(|error| error.to_string())
}
