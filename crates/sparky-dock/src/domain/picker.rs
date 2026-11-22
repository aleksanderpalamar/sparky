use sparky_ui::natural::fold;

use super::app::App;
use super::pinned::Pinned;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Picker {
    query: String,
    chosen: Vec<String>,
}

impl Picker {
    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn search(&mut self, query: String) {
        self.query = query;
    }

    pub fn is_chosen(&self, id: &str) -> bool {
        self.chosen.iter().any(|chosen| chosen == id)
    }

    pub fn toggle(&mut self, id: &str, pinned: &Pinned, room: usize) {
        if pinned.contains(id) {
            return;
        }
        match self.chosen.iter().position(|chosen| chosen == id) {
            Some(index) => {
                self.chosen.remove(index);
            }
            None if self.full(room) => {}
            None => self.chosen.push(id.to_owned()),
        }
    }

    pub fn full(&self, room: usize) -> bool {
        self.chosen.len() >= room
    }

    pub fn chosen(&self) -> &[String] {
        &self.chosen
    }

    pub fn visible<'a>(&self, apps: &'a [App]) -> Vec<&'a App> {
        let wanted = fold(self.query.trim());
        apps.iter()
            .filter(|app| wanted.is_empty() || fold(&app.name).contains(&wanted))
            .collect()
    }
}
