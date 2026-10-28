use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::domain::application::Application;
use crate::domain::entry::Entry;
use crate::domain::icons::Icons;
use crate::domain::properties::Properties;
use crate::domain::thumbnail::Thumbnail;

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

    fn properties(&self, path: &Path) -> io::Result<Properties>;
}

pub trait Opener {
    fn open(&self, path: &Path) -> io::Result<()>;

    fn applications(&self, path: &Path) -> Vec<Application>;

    fn open_with(&self, path: &Path, id: &str) -> io::Result<()>;

    fn email(&self, paths: &[PathBuf]) -> io::Result<()>;
}

pub trait Archiver: Send + Sync {
    fn create(&self, paths: &[PathBuf], target: &Path) -> io::Result<()>;
}

pub trait ThumbnailSource: Send + Sync {
    fn load(&self, path: &Path) -> Option<Thumbnail>;
}

pub trait TrashStore {
    fn open(&self) -> io::Result<PathBuf>;

    fn directory(&self) -> PathBuf;

    fn names(&self) -> io::Result<Vec<String>>;

    fn restore(&self, name: &str) -> io::Result<PathBuf>;
}
