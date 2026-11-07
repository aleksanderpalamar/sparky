use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::config::cli::ConfigSource;
use crate::config::file::read;
use crate::config::settings::Partial;
use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub path: PathBuf,
    pub error: String,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.display().to_string();
        let text = i18n::pairs("error-config", &[("path", &path), ("error", &self.error)]);
        formatter.write_str(&text)
    }
}

pub fn load(source: &ConfigSource, default: &Path) -> Result<Option<Partial>, LoadError> {
    match source {
        ConfigSource::Skip => Ok(None),
        ConfigSource::Default => optional(default),
        ConfigSource::Path(path) => required(path).map(Some),
    }
}

fn optional(path: &Path) -> Result<Option<Partial>, LoadError> {
    match fs::read_to_string(path) {
        Ok(text) => parsed(path, &text).map(Some),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failure(path, error.to_string())),
    }
}

fn required(path: &Path) -> Result<Partial, LoadError> {
    let text = fs::read_to_string(path).map_err(|error| failure(path, error.to_string()))?;
    parsed(path, &text)
}

fn parsed(path: &Path, text: &str) -> Result<Partial, LoadError> {
    read(text).map_err(|error| failure(path, error))
}

fn failure(path: &Path, error: String) -> LoadError {
    LoadError {
        path: path.to_path_buf(),
        error,
    }
}
