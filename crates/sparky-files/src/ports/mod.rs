use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::domain::application::Application;
use crate::domain::entry::Entry;
use crate::domain::icons::Icons;
use crate::domain::image::ImageMime;
use crate::domain::properties::Properties;
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation};
use crate::domain::thumbnail::Thumbnail;

pub trait IconSource {
    fn load(&mut self) -> Icons;

    fn load_sidebar(&mut self) -> Icons {
        self.load()
    }
}

pub trait SidebarSource: Send + Sync {
    fn list(&self, location: SidebarLocation) -> Result<Vec<SidebarItem>, String>;
    fn network_children(&self, uri: &str) -> Result<Vec<SidebarItem>, String>;
    fn open_remote(&self, uri: &str) -> Result<(), String>;
    fn favorite(&self, path: &Path, enabled: bool) -> Result<(), String>;
    fn is_favorite(&self, path: &Path) -> bool;
    fn relocated(&self, from: &Path, to: &Path) -> Result<(), String>;
}

pub trait ImageClipboard {
    fn publish(&self, path: &Path, mime: ImageMime) -> io::Result<()>;
}

pub trait Disk {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>>;

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;

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

#[derive(Debug, Clone, Default)]
pub struct EmptyReport {
    pub removed: usize,
    pub errors: Vec<String>,
}

pub trait TrashStore: Send + Sync {
    fn open(&self) -> io::Result<PathBuf>;

    fn directory(&self) -> PathBuf;

    fn names(&self) -> io::Result<Vec<OsString>>;

    fn restore(&self, name: &OsStr) -> io::Result<PathBuf>;

    fn empty(&self) -> io::Result<EmptyReport>;
}
