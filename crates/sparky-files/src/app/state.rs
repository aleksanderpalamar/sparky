use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sparky_theme::Skin;
use sparky_ui::notice::Notice;
use sparky_ui::viewport::Viewport;

use crate::domain::clip::Clip;
use crate::domain::listing::Listing;
use crate::domain::places::Place;
use crate::ports::{Disk, Opener};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Filter,
    Rename(String),
    Create,
    ConfirmTrash(Vec<String>),
}

pub struct Services {
    pub disk: Box<dyn Disk>,
    pub opener: Box<dyn Opener>,
}

pub struct Setup {
    pub start: PathBuf,
    pub home: PathBuf,
    pub places: Vec<Place>,
    pub skin: Skin,
    pub rows: usize,
}

pub struct Files {
    pub location: PathBuf,
    pub home: PathBuf,
    pub places: Vec<Place>,
    pub listing: Listing,
    pub viewport: Viewport,
    pub stamp: Option<SystemTime>,
    pub mode: Mode,
    pub input: String,
    pub clip: Option<Clip>,
    pub notice: Option<Notice>,
    pub skin: Skin,
    pub services: Services,
}

impl Files {
    pub fn new(setup: Setup, services: Services) -> Self {
        let mut files = Self {
            location: setup.start.clone(),
            home: setup.home,
            places: setup.places,
            listing: Listing::default(),
            viewport: Viewport::new(setup.rows),
            stamp: None,
            mode: Mode::Browse,
            input: String::new(),
            clip: None,
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
        self.listing.chosen().iter().map(|name| self.location.join(name)).collect()
    }

    pub fn follow(&mut self) {
        self.viewport.follow(self.listing.cursor(), self.listing.len());
    }

    pub fn inform(&mut self, text: String) {
        self.notice = Some(Notice::Info(text));
    }

    pub fn fail(&mut self, error: impl Display) {
        self.notice = Some(Notice::Error(error.to_string()));
    }
}
