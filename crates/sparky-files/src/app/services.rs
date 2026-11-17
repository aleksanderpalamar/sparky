use std::path::PathBuf;
use std::sync::Arc;

use sparky_storage::VolumeSource;
use sparky_theme::Skin;

use crate::domain::places::Place;
use crate::ports::{
    Archiver, BookmarkStore, Disk, Dragger, FileClipboard, Finder, IconSource, Opener,
    SidebarSource, ThumbnailSource, Transfer, TrashDisks, TrashStore,
};

pub struct Services {
    pub disk: Box<dyn Disk>,
    pub opener: Box<dyn Opener>,
    pub icons: Box<dyn IconSource>,
    pub volumes: Arc<dyn VolumeSource>,
    pub archiver: Arc<dyn Archiver>,
    pub thumbnails: Arc<dyn ThumbnailSource>,
    pub trash: Arc<dyn TrashStore>,
    pub clipboard: Box<dyn FileClipboard>,
    pub sidebar: Arc<dyn SidebarSource>,
    pub transfer: Arc<dyn Transfer>,
    pub trash_disks: Arc<dyn TrashDisks>,
    pub finder: Arc<dyn Finder>,
    pub dragger: Arc<dyn Dragger>,
    pub bookmarks: Arc<dyn BookmarkStore>,
}

pub struct Setup {
    pub start: PathBuf,
    pub home: PathBuf,
    pub places: Vec<Place>,
    pub skin: Skin,
    pub rows: usize,
}
