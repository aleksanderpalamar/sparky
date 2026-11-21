use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::ports::PinnedStore;

const FILE: &str = "sparky/dock-pinned";

pub struct PinnedFile {
    path: PathBuf,
}

impl PinnedFile {
    pub fn new(config: &Path) -> Self {
        Self {
            path: config.join(FILE),
        }
    }

    pub fn from_env() -> Self {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_default();
        Self::new(&config)
    }
}

impl PinnedStore for PinnedFile {
    fn load(&self) -> Result<Option<Vec<String>>, String> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(Some(contents.lines().map(str::to_owned).collect())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("{}: {error}", self.path.display())),
        }
    }

    fn save(&self, ids: &[String]) -> Result<(), String> {
        let mut contents = ids.join("\n");
        contents.push('\n');
        replace(&self.path, &contents).map_err(|error| format!("{}: {error}", self.path.display()))
    }
}

fn replace(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let staged = path.with_extension("new");
    fs::write(&staged, contents)?;
    fs::rename(&staged, path)
}
