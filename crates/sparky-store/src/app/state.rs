use std::sync::Arc;
use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
};

use sparky_theme::Skin;

use crate::domain::updates::{UpdateReport, UpdateTarget};
use crate::domain::{Action, Application, SearchSource};
use crate::ports::StoreBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Browse,
    Search,
    Installed,
    Updates,
}

#[derive(Debug)]
pub struct Store {
    pub skin: Skin,
    pub backend: Arc<dyn StoreBackend>,
    pub query: String,
    pub source: SearchSource,
    pub page: Page,
    pub apps: Vec<Application>,
    pub icons: HashMap<String, PathBuf>,
    pub selected: Option<usize>,
    pub pending: Option<(Application, Action)>,
    pub visible_count: usize,
    pub feature_index: usize,
    pub busy: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub flathub_missing: bool,
    pub updates: Option<UpdateReport>,
    pub update_pending: Option<Vec<UpdateTarget>>,
    pub update_queue: VecDeque<UpdateTarget>,
    pub updating: Option<UpdateTarget>,
    pub update_failures: Vec<String>,
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
            icons: HashMap::new(),
            selected: None,
            pending: None,
            visible_count: 80,
            feature_index: 0,
            busy: false,
            error: None,
            notice: None,
            flathub_missing: false,
            updates: None,
            update_pending: None,
            update_queue: VecDeque::new(),
            updating: None,
            update_failures: Vec::new(),
        }
    }

    pub fn selected_app(&self) -> Option<&Application> {
        self.selected.and_then(|index| self.apps.get(index))
    }
}
