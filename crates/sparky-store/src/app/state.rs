use std::sync::Arc;

use sparky_theme::Skin;

use crate::domain::{Action, Application, SearchSource};
use crate::ports::StoreBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Browse,
    Search,
    Installed,
}

#[derive(Debug)]
pub struct Store {
    pub skin: Skin,
    pub backend: Arc<dyn StoreBackend>,
    pub query: String,
    pub source: SearchSource,
    pub page: Page,
    pub apps: Vec<Application>,
    pub selected: Option<usize>,
    pub pending: Option<(Application, Action)>,
    pub visible_count: usize,
    pub busy: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub flathub_missing: bool,
}

impl Store {
    pub fn new(backend: Arc<dyn StoreBackend>, skin: Skin) -> Self {
        Self {
            skin,
            backend,
            query: String::new(),
            source: SearchSource::Preferred,
            page: Page::Browse,
            apps: Vec::new(),
            selected: None,
            pending: None,
            visible_count: 80,
            busy: false,
            error: None,
            notice: None,
            flathub_missing: false,
        }
    }

    pub fn selected_app(&self) -> Option<&Application> {
        self.selected.and_then(|index| self.apps.get(index))
    }
}
