use std::collections::HashMap;

use iced::window;
use sparky_launcher::app::pictures::Shown;
use sparky_theme::Skin;

use crate::domain::app::App;
use crate::domain::drag::Drag;
use crate::domain::layout::{Geometry, Layout, Visibility};
use crate::domain::motion::Ease;
use crate::domain::picker::Picker;
use crate::domain::pinned::Pinned;
use crate::domain::presence::{Presence, Seen, presence};
use crate::domain::visibility::{Moment, visibility};
use crate::ports::{PinnedStore, Starter};

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
    store: Box<dyn PinnedStore>,
    starter: Box<dyn Starter>,
}

impl Dock {
    pub fn new(skin: Skin, store: Box<dyn PinnedStore>, starter: Box<dyn Starter>) -> Self {
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
            store,
            starter,
        }
    }

    pub fn layout(&self) -> Layout {
        Layout::new(self.preferences.slot())
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
        let layout = self.layout();
        let autohide = self.preferences.autohide;
        if self.tucked() {
            return Some(layout.geometry(count, Visibility::Hidden, autohide));
        }
        let mut geometry = layout.geometry(count, Visibility::Shown, autohide);
        if self.hovering || self.zoom.iter().any(|ease| !ease.settled()) {
            geometry = geometry.lifted(&layout);
        }
        match self.menu {
            Some(_) => Some(geometry.with_menu()),
            None => Some(geometry),
        }
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
