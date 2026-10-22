use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::domain::entry::Entry;
use crate::domain::icons::Icons;

pub trait IconSource {
    fn load(&mut self) -> Icons;
}

pub trait Disk {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>>;

    fn exists(&self, path: &Path) -> bool;

    fn stamp(&self, dir: &Path) -> Option<SystemTime>;

    fn create_dir(&self, path: &Path) -> io::Result<()>;

    fn create_file(&self, path: &Path) -> io::Result<()>;

    fn relocate(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn trash(&self, path: &Path) -> io::Result<()>;
}

pub trait Opener {
    fn open(&self, path: &Path) -> io::Result<()>;
}
