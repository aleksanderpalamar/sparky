use std::collections::BTreeSet;

use crate::domain::entry::{Entry, sort};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Hidden {
    #[default]
    Concealed,
    Shown,
}

impl Hidden {
    pub fn toggled(self) -> Self {
        match self {
            Hidden::Concealed => Hidden::Shown,
            Hidden::Shown => Hidden::Concealed,
        }
    }

    fn allows(self, entry: &Entry) -> bool {
        self == Hidden::Shown || !entry.hidden()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Listing {
    entries: Vec<Entry>,
    shown: Vec<usize>,
    hidden: Hidden,
    filter: String,
    marked: BTreeSet<String>,
    cursor: usize,
}

impl Listing {
    pub fn new(mut entries: Vec<Entry>, hidden: Hidden) -> Self {
        sort(&mut entries);
        let mut listing = Self {
            entries,
            hidden,
            ..Self::default()
        };
        listing.refresh();
        listing
    }

    pub fn replace(&mut self, mut entries: Vec<Entry>) {
        let selected = self.selected_name();
        sort(&mut entries);
        self.marked
            .retain(|name| entries.iter().any(|entry| &entry.name == name));
        self.entries = entries;
        self.refresh();
        if let Some(name) = selected {
            self.select_name(&name);
        }
    }

    pub fn visible(&self) -> impl Iterator<Item = &Entry> {
        self.shown
            .iter()
            .filter_map(|index| self.entries.get(*index))
    }

    pub fn len(&self) -> usize {
        self.shown.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shown.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&Entry> {
        self.entries.get(*self.shown.get(index)?)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selected(&self) -> Option<&Entry> {
        self.get(self.cursor)
    }

    pub fn move_by(&mut self, delta: isize) {
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(self.len().saturating_sub(1));
    }

    pub fn first(&mut self) {
        self.cursor = 0;
    }

    pub fn last(&mut self) {
        self.cursor = self.len().saturating_sub(1);
    }

    pub fn select(&mut self, index: usize) {
        if index < self.len() {
            self.cursor = index;
        }
    }

    pub fn select_name(&mut self, name: &str) {
        let found = self.visible().position(|entry| entry.name == name);
        if let Some(index) = found {
            self.cursor = index;
        }
    }

    pub fn hidden(&self) -> Hidden {
        self.hidden
    }

    pub fn toggle_hidden(&mut self) {
        let selected = self.selected_name();
        self.hidden = self.hidden.toggled();
        self.refresh();
        if let Some(name) = selected {
            self.select_name(&name);
        }
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_owned();
        self.cursor = 0;
        self.refresh();
    }

    pub fn toggle_mark(&mut self) {
        let Some(name) = self.selected_name() else {
            return;
        };
        if !self.marked.remove(&name) {
            self.marked.insert(name);
        }
    }

    pub fn is_marked(&self, entry: &Entry) -> bool {
        self.marked.contains(&entry.name)
    }

    pub fn marked_count(&self) -> usize {
        self.marked.len()
    }

    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    pub fn select_all(&mut self) {
        self.marked = self.visible().map(|entry| entry.name.clone()).collect();
    }

    pub fn chosen(&self) -> Vec<String> {
        match self.marked.is_empty() {
            true => self.selected_name().into_iter().collect(),
            false => self.marked.iter().cloned().collect(),
        }
    }

    fn selected_name(&self) -> Option<String> {
        self.selected().map(|entry| entry.name.clone())
    }

    fn refresh(&mut self) {
        let needle = self.filter.to_lowercase();
        self.shown = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| self.hidden.allows(entry))
            .filter(|(_, entry)| entry.name.to_lowercase().contains(&needle))
            .map(|(index, _)| index)
            .collect();
        self.cursor = self.cursor.min(self.len().saturating_sub(1));
    }
}
