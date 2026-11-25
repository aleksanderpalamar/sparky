use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::ports::Files;

#[derive(Default)]
pub struct FakeFiles {
    files: BTreeMap<PathBuf, String>,
    broken: BTreeMap<PathBuf, io::ErrorKind>,
}

impl FakeFiles {
    pub fn with(mut self, path: &str, text: &str) -> Self {
        self.files.insert(PathBuf::from(path), text.to_owned());
        self
    }

    pub fn broken(mut self, path: &str, kind: io::ErrorKind) -> Self {
        self.broken.insert(PathBuf::from(path), kind);
        self
    }

    fn failure(&self, path: &Path) -> io::Result<()> {
        match self.broken.get(path) {
            Some(kind) => Err(io::Error::from(*kind)),
            None => Ok(()),
        }
    }
}

impl Files for FakeFiles {
    fn read(&self, path: &Path) -> io::Result<String> {
        self.failure(path)?;
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn entries(&self, directory: &Path) -> io::Result<Vec<String>> {
        self.failure(directory)?;
        let mut names: Vec<String> = self
            .files
            .keys()
            .filter_map(|path| path.strip_prefix(directory).ok())
            .filter_map(|rest| rest.components().next())
            .map(|first| first.as_os_str().to_string_lossy().into_owned())
            .collect();
        names.dedup();
        match names.is_empty() {
            true => Err(io::Error::from(io::ErrorKind::NotFound)),
            false => Ok(names),
        }
    }
}
