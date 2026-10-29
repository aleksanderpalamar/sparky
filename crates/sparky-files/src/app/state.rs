use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use iced::{Point, Size};
use sparky_storage::{Operation, Volume, VolumeSource};
use sparky_theme::Skin;
use sparky_ui::notice::Notice;
use sparky_ui::viewport::Viewport;

use crate::app::previews::Previews;
use crate::domain::application::Application;
use crate::domain::clip::Clip;
use crate::domain::context::Target;
use crate::domain::entry::Kind;
use crate::domain::grid::{Layout, ViewMode};
use crate::domain::icons::Icons;
use crate::domain::listing::Listing;
use crate::domain::places::Place;
use crate::domain::properties::Properties;
use crate::ports::{
    Archiver, Disk, IconSource, ImageClipboard, Opener, ThumbnailSource, TrashStore,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Filter,
    Rename(String),
    Create,
    New(Kind),
    Destination(Clip),
    Compress(Vec<PathBuf>),
    ConfirmTrash(Vec<String>),
}

pub struct Services {
    pub disk: Box<dyn Disk>,
    pub opener: Box<dyn Opener>,
    pub icons: Box<dyn IconSource>,
    pub volumes: Arc<dyn VolumeSource>,
    pub archiver: Arc<dyn Archiver>,
    pub thumbnails: Arc<dyn ThumbnailSource>,
    pub trash: Box<dyn TrashStore>,
    pub image_clipboard: Box<dyn ImageClipboard>,
}

pub struct Setup {
    pub start: PathBuf,
    pub home: PathBuf,
    pub places: Vec<Place>,
    pub skin: Skin,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub target: Target,
    pub name: Option<String>,
    pub point: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWith {
    pub path: PathBuf,
    pub apps: Vec<Application>,
}

pub struct Files {
    pub location: PathBuf,
    pub home: PathBuf,
    pub places: Vec<Place>,
    pub listing: Listing,
    pub viewport: Viewport,
    pub view_mode: ViewMode,
    pub layout: Layout,
    pub size: Size,
    pub icons: Icons,
    pub volumes: Vec<Volume>,
    pub volume_work: Option<Operation>,
    pub volume_error: Option<String>,
    pub stamp: Option<SystemTime>,
    pub mode: Mode,
    pub input: String,
    pub clip: Option<Clip>,
    pub pointer: Point,
    pub context: Option<ContextMenu>,
    pub details: Option<Properties>,
    pub open_with: Option<OpenWith>,
    pub archive_work: Option<PathBuf>,
    pub previews: Previews,
    pub notice: Option<Notice>,
    pub skin: Skin,
    pub services: Services,
}

impl Files {
    pub fn new(setup: Setup, mut services: Services) -> Self {
        let icons = services.icons.load();
        let mut files = Self {
            location: setup.start.clone(),
            home: setup.home,
            places: setup.places,
            listing: Listing::default(),
            viewport: Viewport::new(setup.rows),
            view_mode: ViewMode::Grid,
            layout: Layout::new(1, setup.rows),
            size: Size::new(
                crate::view::style::WINDOW_WIDTH,
                crate::view::style::WINDOW_HEIGHT,
            ),
            icons,
            volumes: Vec::new(),
            volume_work: None,
            volume_error: None,
            stamp: None,
            mode: Mode::Browse,
            input: String::new(),
            clip: None,
            pointer: Point::ORIGIN,
            context: None,
            details: None,
            open_with: None,
            archive_work: None,
            previews: Previews::default(),
            notice: None,
            skin: setup.skin,
            services,
        };
        files.go(setup.start);
        files
    }

    pub fn shown(&self, path: &Path) -> String {
        sparky_xdg::tilde(path, &self.home)
    }

    pub fn chosen_paths(&self) -> Vec<PathBuf> {
        self.listing
            .chosen()
            .iter()
            .map(|name| self.location.join(name))
            .collect()
    }

    pub fn follow(&mut self) {
        self.viewport.follow(
            self.listing.cursor() / self.layout.columns,
            self.layout.row_count(self.listing.len()),
        );
    }

    pub fn inform(&mut self, text: String) {
        self.notice = Some(Notice::Info(text));
    }

    pub fn fail(&mut self, error: impl Display) {
        self.notice = Some(Notice::Error(error.to_string()));
    }
}
