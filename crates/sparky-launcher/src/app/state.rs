use std::collections::HashMap;

use sparky_theme::Skin;
use sparky_ui::viewport::Viewport;

use crate::app::pictures::Shown;
use crate::app::wheel::Wheel;
use crate::domain::app::App;
use crate::domain::groups::Groups;
use crate::domain::mode::Mode;
use crate::domain::overlay::Overlay;
use crate::domain::search::rank;
use crate::domain::shelf::{Shelf, shelves};
use crate::ports::GroupStore;

pub struct Launcher {
    pub apps: Vec<App>,
    pub query: String,
    pub cursor: usize,
    pub list: Viewport,
    pub skin: Skin,
    pub mode: Mode,
    pub icons: HashMap<String, Shown>,
    pub shelf: Shelf,
    pub groups: Groups,
    pub overlay: Overlay,
    pub pointer: (f32, f32),
    pub wheel: Wheel,
    store: Box<dyn GroupStore>,
}

impl Launcher {
    pub fn new(apps: Vec<App>, skin: Skin, mode: Mode, store: Box<dyn GroupStore>) -> Self {
        let saved = store.load().unwrap_or_else(|error| {
            eprintln!("sparky-launcher: não deu para ler os grupos: {error}");
            Vec::new()
        });
        Self {
            apps,
            query: String::new(),
            cursor: 0,
            list: Viewport::new(mode.rows()),
            skin,
            mode,
            icons: HashMap::new(),
            shelf: Shelf::Home,
            groups: Groups::new(saved),
            overlay: Overlay::Closed,
            pointer: (0.0, 0.0),
            wheel: Wheel::default(),
            store,
        }
    }

    pub fn results(&self) -> Vec<&App> {
        let order = self.mode.order();
        match self.query.trim().is_empty() {
            true => rank(
                self.apps
                    .iter()
                    .filter(|app| self.shelf.holds(app, &self.groups)),
                "",
                order,
            ),
            false => rank(&self.apps, &self.query, order),
        }
    }

    pub fn shelves(&self) -> Vec<Shelf> {
        match self.mode.has_shelves() {
            true => shelves(&self.apps, &self.groups),
            false => Vec::new(),
        }
    }

    pub fn chosen(&self) -> Option<App> {
        self.results().get(self.cursor).map(|app| (*app).clone())
    }

    pub fn pick(&self, index: usize) -> Option<App> {
        self.results().get(index).map(|app| (*app).clone())
    }

    pub fn persist(&self) {
        if let Err(error) = self.store.save(self.groups.list()) {
            eprintln!("sparky-launcher: não deu para salvar os grupos: {error}");
        }
    }
}
