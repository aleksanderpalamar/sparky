use std::collections::HashMap;

use iced::window;
use sparky_launcher::app::pictures::Shown;
use sparky_theme::Skin;

use crate::domain::app::App;
use crate::domain::layout::{Geometry, Layout, Visibility};
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

    pub fn desired(&self) -> Option<Geometry> {
        if !self.preferences.enabled || !self.catalogued {
            return None;
        }
        let count = self.pinned.resolve(&self.apps).len();
        let geometry = self
            .layout()
            .geometry(count, self.visibility(), self.preferences.autohide);
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
