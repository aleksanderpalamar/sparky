use std::collections::HashMap;

use iced::window;
use sparky_launcher::app::pictures::Shown;
use sparky_theme::Skin;

use crate::domain::app::App;
use crate::domain::picker::Picker;
use crate::domain::pinned::Pinned;
use crate::domain::presence::{Presence, Seen, presence};
use crate::ports::{PinnedStore, Starter};

pub struct OpenPicker {
    pub window: window::Id,
    pub picker: Picker,
}

pub struct Dock {
    pub skin: Skin,
    pub window: window::Id,
    pub shown: bool,
    pub apps: Vec<App>,
    pub pinned: Pinned,
    pub windows: Vec<Seen>,
    pub icons: HashMap<String, Shown>,
    pub picker: Option<OpenPicker>,
    store: Box<dyn PinnedStore>,
    starter: Box<dyn Starter>,
}

impl Dock {
    pub fn new(skin: Skin, store: Box<dyn PinnedStore>, starter: Box<dyn Starter>) -> Self {
        let pinned = restore(store.as_ref());
        Self {
            skin,
            window: window::Id::unique(),
            shown: false,
            apps: Vec::new(),
            pinned,
            windows: Vec::new(),
            icons: HashMap::new(),
            picker: None,
            store,
            starter,
        }
    }

    pub fn entries(&self) -> Vec<(&App, Presence)> {
        self.pinned
            .resolve(&self.apps)
            .into_iter()
            .map(|app| (app, presence(app, &self.windows)))
            .collect()
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

    pub fn pin(&mut self, ids: &[String]) -> bool {
        if !self.pinned.add(ids.iter().cloned()) {
            return false;
        }
        if let Err(error) = self.store.save(self.pinned.ids()) {
            eprintln!("sparky-dock: não salvou o dock: {error}");
        }
        true
    }
}

fn restore(store: &dyn PinnedStore) -> Pinned {
    match store.load() {
        Ok(Some(ids)) => {
            let pinned = Pinned::from_ids(ids);
            match pinned.ids().is_empty() {
                true => Pinned::default(),
                false => pinned,
            }
        }
        Ok(None) => Pinned::default(),
        Err(error) => {
            eprintln!("sparky-dock: configuração ilegível, usando a padrão: {error}");
            Pinned::default()
        }
    }
}
