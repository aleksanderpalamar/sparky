use std::collections::HashMap;

use sparky_theme::Skin;
use sparky_ui::viewport::Viewport;

use crate::app::pictures::Shown;
use crate::domain::app::App;
use crate::domain::grid::Window;
use crate::domain::library::Library;
use crate::domain::mode::Mode;
use crate::domain::overlay::Overlay;
use crate::domain::search::rank;
use crate::domain::shelf::{Shelf, shelves};
use crate::ports::LibraryStore;

pub struct Launcher {
    pub apps: Vec<App>,
    pub query: String,
    pub cursor: Option<usize>,
    pub list: Viewport,
    pub window: Window,
    pub skin: Skin,
    pub mode: Mode,
    pub icons: HashMap<String, Shown>,
    pub shelf: Shelf,
    pub library: Library,
    pub overlay: Overlay,
    pub pointer: (f32, f32),
    store: Box<dyn LibraryStore>,
}

impl Launcher {
    pub fn new(apps: Vec<App>, skin: Skin, mode: Mode, store: Box<dyn LibraryStore>) -> Self {
        let library = store.load().unwrap_or_else(|error| {
            eprintln!("sparky-launcher: não deu para ler os grupos: {error}");
            Library::default()
        });
        Self {
            apps,
            query: String::new(),
            cursor: mode.first_selection(),
            list: Viewport::new(mode.rows()),
            window: Window::default(),
            skin,
            mode,
            icons: HashMap::new(),
            shelf: Shelf::Home,
            library,
            overlay: Overlay::Closed,
            pointer: (0.0, 0.0),
            store,
        }
    }

    pub fn results(&self) -> Vec<&App> {
        let order = self.mode.order();
        match self.query.trim().is_empty() {
            true => rank(
                self.apps
                    .iter()
                    .filter(|app| !self.mode.has_shelves() || self.shelf.holds(app, &self.library)),
                "",
                order,
            ),
            false => rank(&self.apps, &self.query, order),
        }
    }

    pub fn shelves(&self) -> Vec<Shelf> {
        match self.mode.has_shelves() {
            true => shelves(&self.apps, &self.library),
            false => Vec::new(),
        }
    }

    pub fn chosen(&self) -> Option<App> {
        let cursor = self.cursor?;
        self.results().get(cursor).map(|app| (*app).clone())
    }

    pub fn pick(&self, index: usize) -> Option<App> {
        self.results().get(index).map(|app| (*app).clone())
    }

    pub fn find(&self, id: &str) -> Option<App> {
        self.apps.iter().find(|app| app.id == id).cloned()
    }

    pub fn persist(&self) {
        if let Err(error) = self.store.save(&self.library) {
            eprintln!("sparky-launcher: não deu para salvar os grupos: {error}");
        }
    }
}
