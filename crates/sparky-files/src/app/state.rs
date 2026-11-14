use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use iced::{Point, Size};
use sparky_storage::{Operation, Volume};
use sparky_theme::Skin;
use sparky_ui::notice::Notice;
use sparky_ui::viewport::Viewport;

use crate::app::bookmarks::BookmarkBar;
use crate::app::drop::Gesture;
use crate::app::jobs::Job;
pub use crate::app::mode::Mode;
use crate::app::previews::Previews;
use crate::app::search::Search;
pub use crate::app::services::{Services, Setup};
use crate::app::tabs::Tab;
use crate::domain::application::{AppChoices, Remember};
use crate::domain::clip::Clip;
use crate::domain::context::Target;
use crate::domain::drop::Override;
use crate::domain::grid::{Layout, ViewMode};
use crate::domain::icons::Icons;
use crate::domain::listing::Listing;
use crate::domain::places::Place;
use crate::domain::properties::Properties;
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation};
use crate::domain::tabs::Tabs;
use crate::domain::toast::Toast;
use crate::domain::undo::Journal;
use crate::ports::TrashStore;

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub target: Target,
    pub name: Option<String>,
    pub point: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWith {
    pub path: PathBuf,
    pub choices: AppChoices,
    pub query: String,
    pub selected: Option<String>,
    pub remember: Remember,
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
    pub sidebar_icons: Icons,
    pub sidebar_page: Option<SidebarLocation>,
    pub network_uri: Option<String>,
    pub network_pending: Option<String>,
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,
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
    pub emptying: bool,
    pub job: Option<Job>,
    pub search: Option<Search>,
    pub tabs: Tabs<Tab>,
    pub gesture: Gesture,
    pub held: Override,
    pub journal: Journal,
    pub other_trashes: Vec<Arc<dyn TrashStore>>,
    pub previews: Previews,
    pub notice: Option<Notice>,
    pub toast: Option<Toast>,
    pub bookmarks: BookmarkBar,
    pub skin: Skin,
    pub services: Services,
}

impl Files {
    pub fn new(setup: Setup, mut services: Services) -> Self {
        let icons = services.icons.load();
        let sidebar_icons = services.icons.load_sidebar();
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
            sidebar_icons,
            sidebar_page: None,
            network_uri: None,
            network_pending: None,
            sidebar_items: Vec::new(),
            sidebar_cursor: 0,
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
            emptying: false,
            job: None,
            search: None,
            tabs: Tabs::default(),
            gesture: Gesture::Idle,
            held: Override::None,
            journal: Journal::default(),
            other_trashes: Vec::new(),
            previews: Previews::default(),
            notice: None,
            toast: None,
            bookmarks: BookmarkBar::default(),
            skin: setup.skin,
            services,
        };
        files.go(setup.start);
        files.reload_bookmarks();
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
