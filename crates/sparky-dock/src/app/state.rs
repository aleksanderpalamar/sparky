use std::collections::HashMap;

use iced::window;
use sparky_launcher::app::pictures::Shown;
use sparky_theme::Skin;

use crate::domain::app::App;
use crate::domain::drag::Drag;
use crate::domain::fit::{capacity, fitted};
use crate::domain::layout::{Geometry, Layout, Visibility};
use crate::domain::motion::Ease;
use crate::domain::picker::Picker;
use crate::domain::pinned::Pinned;
use crate::domain::presence::{Presence, Seen, presence};
use crate::domain::region::{Rect, Stage, region};
use crate::domain::visibility::{Moment, visibility};
use crate::ports::{GlassBoard, PinnedStore, Starter};

pub struct OpenPicker {
    pub window: window::Id,
    pub picker: Picker,
}

pub struct Dock {
    pub skin: Skin,
    pub window: window::Id,
    pub surface: Option<Geometry>,
    pub preferences: sparky_config::Dock,
    pub catalogued: bool,
    pub apps: Vec<App>,
    pub pinned: Pinned,
    pub windows: Vec<Seen>,
    pub icons: HashMap<String, Shown>,
    pub picker: Option<OpenPicker>,
    pub hovering: bool,
    pub menu: Option<String>,
    pub pointer: Option<iced::Point>,
    pub drag: Option<Drag>,
    pub zoom: Vec<Ease>,
    pub slide: Ease,
    pub clock: Option<std::time::Instant>,
    pub accepting: Option<Rect>,
    pub glass: Option<sparky_ipc::GlassArea>,
    pub output: Option<u32>,
    store: Box<dyn PinnedStore>,
    starter: Box<dyn Starter>,
    board: Box<dyn GlassBoard>,
}

impl Dock {
    pub fn new(
        skin: Skin,
        store: Box<dyn PinnedStore>,
        starter: Box<dyn Starter>,
        board: Box<dyn GlassBoard>,
    ) -> Self {
        let pinned = super::restore::restore(store.as_ref());
        Self {
            skin,
            window: window::Id::unique(),
            surface: None,
            preferences: sparky_config::Dock::default(),
            catalogued: false,
            apps: Vec::new(),
            pinned,
            windows: Vec::new(),
            icons: HashMap::new(),
            picker: None,
            hovering: false,
            menu: None,
            pointer: None,
            drag: None,
            zoom: Vec::new(),
            slide: Ease::at(0.0),
            clock: None,
            accepting: None,
            glass: None,
            output: None,
            store,
            starter,
            board,
        }
    }

    pub fn layout(&self) -> Layout {
        let count = self.pinned.resolve(&self.apps).len();
        fitted(self.preferences.slot(), count, self.output)
    }

    pub fn corner(&self) -> f32 {
        self.preferences.corner() as f32
    }

    pub fn room(&self) -> usize {
        capacity(self.output).saturating_sub(self.pinned.resolve(&self.apps).len())
    }

    pub fn entries(&self) -> Vec<(&App, Presence)> {
        self.pinned
            .resolve(&self.apps)
            .into_iter()
            .map(|app| (app, presence(app, &self.windows)))
            .collect()
    }

    pub fn visibility(&self) -> Visibility {
        visibility(&Moment {
            autohide: self.preferences.autohide,
            windows: &self.windows,
            hovering: self.hovering,
            busy: self.picker.is_some() || self.menu.is_some(),
        })
    }

    pub fn tucked(&self) -> bool {
        self.visibility() == Visibility::Hidden && self.slide.settled() && self.slide.value == 1.0
    }

    pub fn desired(&self) -> Option<Geometry> {
        if !self.preferences.enabled || !self.catalogued {
            return None;
        }
        let count = self.pinned.resolve(&self.apps).len();
        Some(self.layout().frame(count, self.preferences.autohide))
    }

    pub fn stage(&self) -> Stage {
        match (self.tucked(), self.menu.is_some(), self.hovering) {
            (true, _, _) => Stage::Tucked,
            (false, true, _) => Stage::Menu,
            (false, false, true) => Stage::Lifted,
            (false, false, false) => Stage::Resting,
        }
    }

    pub fn region(&self) -> Option<Rect> {
        let frame = self.surface?;
        let count = self.pinned.resolve(&self.apps).len();
        Some(region(&self.layout(), count, frame, self.stage()))
    }

    pub fn start(&self, id: &str) {
        let Some(app) = self.apps.iter().find(|app| app.id == id) else {
            eprintln!("sparky-dock: {id} não está instalado");
            return;
        };
        if let Err(error) = self.starter.start(&app.launch) {
            eprintln!("sparky-dock: não abriu {}: {error}", app.name);
        }
    }

    pub fn pin(&mut self, ids: &[String]) {
        if self.pinned.add(ids.iter().cloned()) {
            self.persist();
        }
    }

    pub fn show_glass(&self, area: sparky_ipc::GlassArea) {
        self.board.announce(area);
    }

    pub fn place(&mut self, id: &str, before: Option<&str>) {
        if self.pinned.place(id, before) {
            self.persist();
        }
    }

    pub fn unpin(&mut self, id: &str) {
        if self.pinned.remove(id) {
            self.persist();
        }
    }

    fn persist(&self) {
        if let Err(error) = self.store.save(self.pinned.ids()) {
            eprintln!("sparky-dock: não salvou o dock: {error}");
        }
    }
}
