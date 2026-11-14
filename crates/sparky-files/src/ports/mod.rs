use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::SystemTime;

use crate::domain::application::{AppChoices, Remember};
use crate::domain::archive::Extraction;
use crate::domain::bookmarks::Bookmark;
use crate::domain::clip::{ClipOffer, Pasted};
use crate::domain::entry::{Entry, Meta};
use crate::domain::icons::Icons;
use crate::domain::listing::Hidden;
use crate::domain::properties::Properties;
use crate::domain::search::{Finish, Query};
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation};
use crate::domain::thumbnail::Thumbnail;
use crate::domain::transfer::{Progress, Step};
use crate::domain::undo::TrashEntry;

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
    fn relocated(&self, moves: &[(PathBuf, PathBuf)]) -> Result<(), String>;
}

pub trait BookmarkStore: Send + Sync {
    fn load(&self) -> Result<Vec<Bookmark>, String>;
    fn save(&self, bookmarks: &[Bookmark]) -> Result<(), String>;
}

pub trait FileClipboard {
    fn publish(&self, offer: &ClipOffer) -> io::Result<()>;

    fn clear(&self) -> io::Result<()>;

    fn read(&self) -> Pasted;
}

pub trait Disk {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>>;

    fn detail(&self, path: &Path) -> Meta;

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;

    fn exists(&self, path: &Path) -> bool;

    fn stamp(&self, dir: &Path) -> Option<SystemTime>;

    fn create_dir(&self, path: &Path) -> io::Result<()>;

    fn create_file(&self, path: &Path) -> io::Result<()>;

    fn relocate(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn trash(&self, path: &Path) -> io::Result<TrashEntry>;

    fn delete(&self, path: &Path) -> io::Result<()>;

    fn has_trash(&self, dir: &Path) -> bool;

    fn device(&self, path: &Path) -> Option<u64>;

    fn untrash(&self, entry: &TrashEntry) -> io::Result<PathBuf>;

    fn properties(&self, path: &Path) -> io::Result<Properties>;
}

pub trait Opener {
    fn open(&self, path: &Path) -> io::Result<()>;

    fn applications(&self, path: &Path) -> AppChoices;

    fn open_with(&self, path: &Path, id: &str, remember: Remember) -> io::Result<()>;

    fn email(&self, paths: &[PathBuf]) -> io::Result<()>;
}

pub trait Archiver: Send + Sync {
    fn create(&self, paths: &[PathBuf], target: &Path) -> io::Result<()>;

    fn extract(
        &self,
        archive: &Path,
        into: &Path,
        advance: &mut dyn FnMut(Progress) -> bool,
    ) -> io::Result<Extraction>;
}

pub trait Dragger: Send + Sync {
    fn start(&self, uris: Vec<String>) -> Result<(), String>;
}

pub trait Finder: Send + Sync {
    fn find(
        &self,
        root: &Path,
        query: &Query,
        hidden: Hidden,
        cancel: &AtomicBool,
        found: &mut dyn FnMut(Vec<Entry>),
    ) -> Finish;
}

pub trait Transfer: Send + Sync {
    fn size(&self, path: &Path) -> u64;

    fn run(
        &self,
        step: &Step,
        advance: &mut dyn FnMut(u64) -> bool,
    ) -> io::Result<Option<TrashEntry>>;
}

pub trait ThumbnailSource: Send + Sync {
    fn load(&self, path: &Path) -> Option<Thumbnail>;
}

#[derive(Debug, Clone, Default)]
pub struct EmptyReport {
    pub removed: usize,
    pub errors: Vec<String>,
}

pub trait TrashDisks: Send + Sync {
    fn others(&self) -> Vec<std::sync::Arc<dyn TrashStore>>;
}

pub trait TrashStore: Send + Sync {
    fn open(&self) -> io::Result<PathBuf>;

    fn directory(&self) -> PathBuf;

    fn label(&self) -> Option<String> {
        None
    }

    fn names(&self) -> io::Result<Vec<OsString>>;

    fn restore(&self, name: &OsStr) -> io::Result<PathBuf>;

    fn empty(&self) -> io::Result<EmptyReport>;
}
