use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::linux::host::{Host, Ran, Usage};

#[derive(Default)]
pub struct Fake {
    files: HashMap<PathBuf, io::Result<String>>,
    dirs: HashMap<PathBuf, Vec<PathBuf>>,
    vars: HashMap<String, String>,
    usage: Option<Usage>,
    commands: HashMap<String, Ran>,
}

impl Fake {
    pub fn file(mut self, path: &str, text: &str) -> Self {
        self.files.insert(PathBuf::from(path), Ok(text.to_owned()));
        self
    }

    pub fn broken(mut self, path: &str) -> Self {
        let error = io::Error::from(ErrorKind::PermissionDenied);
        self.files.insert(PathBuf::from(path), Err(error));
        self
    }

    pub fn dir(mut self, path: &str, names: &[&str]) -> Self {
        let root = PathBuf::from(path);
        let entries = names.iter().map(|name| root.join(name)).collect();
        self.dirs.insert(root, entries);
        self
    }

    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_owned(), value.to_owned());
        self
    }

    pub fn disk(mut self, total: u64, free: u64) -> Self {
        self.usage = Some(Usage { total, free });
        self
    }

    pub fn command(mut self, program: &str, ran: Ran) -> Self {
        self.commands.insert(program.to_owned(), ran);
        self
    }
}

impl Host for Fake {
    fn read(&self, path: &Path) -> io::Result<String> {
        match self.files.get(path) {
            Some(Ok(text)) => Ok(text.clone()),
            Some(Err(error)) => Err(io::Error::from(error.kind())),
            None => Err(io::Error::from(ErrorKind::NotFound)),
        }
    }

    fn list(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        self.dirs
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))
    }

    fn var(&self, name: &str) -> Option<String> {
        self.vars.get(name).cloned()
    }

    fn usage(&self, _: &Path) -> io::Result<Usage> {
        self.usage
            .ok_or_else(|| io::Error::from(ErrorKind::NotFound))
    }

    fn run(&self, program: &str, _: &[&str], _: Duration) -> Ran {
        self.commands.get(program).cloned().unwrap_or(Ran::Missing)
    }
}
