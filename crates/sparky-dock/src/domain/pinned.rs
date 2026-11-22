use super::app::App;

pub const NATIVE: [&str; 4] = [
    "sparky-settings",
    "sparky-files",
    "sparky-terminal",
    "sparky-store",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pinned {
    ids: Vec<String>,
}

impl Default for Pinned {
    fn default() -> Self {
        Self::from_ids(NATIVE.iter().map(|id| (*id).to_owned()))
    }
}

impl Pinned {
    pub fn from_ids(ids: impl IntoIterator<Item = String>) -> Self {
        let mut pinned = Self { ids: Vec::new() };
        pinned.add(ids);
        pinned
    }

    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|pinned| pinned == id)
    }

    pub fn resolve<'a>(&self, apps: &'a [App]) -> Vec<&'a App> {
        self.ids
            .iter()
            .filter_map(|id| apps.iter().find(|app| &app.id == id))
            .collect()
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.ids.len();
        self.ids.retain(|pinned| pinned != id);
        self.ids.len() != before
    }

    pub fn place(&mut self, id: &str, before: Option<&str>) -> bool {
        let Some(from) = self.ids.iter().position(|pinned| pinned == id) else {
            return false;
        };
        let item = self.ids.remove(from);
        let to = before
            .and_then(|next| self.ids.iter().position(|pinned| pinned == next))
            .unwrap_or(self.ids.len());
        self.ids.insert(to, item);
        to != from
    }

    pub fn add(&mut self, ids: impl IntoIterator<Item = String>) -> bool {
        let before = self.ids.len();
        for id in ids {
            let id = id.trim();
            if valid(id) && !self.contains(id) {
                self.ids.push(id.to_owned());
            }
        }
        self.ids.len() != before
    }
}

fn valid(id: &str) -> bool {
    !id.is_empty()
        && !id
            .chars()
            .any(|letter| letter.is_control() || letter == '/')
}
