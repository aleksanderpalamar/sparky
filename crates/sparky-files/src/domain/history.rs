use std::path::PathBuf;

use crate::domain::sidebar_location::SidebarLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visit {
    Folder(PathBuf),
    Sidebar(SidebarLocation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History<T> {
    back: Vec<T>,
    forward: Vec<T>,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            back: Vec::new(),
            forward: Vec::new(),
        }
    }
}

impl<T> History<T> {
    pub fn visit(&mut self, left: T) {
        self.back.push(left);
        self.forward.clear();
    }

    pub fn previous(&self) -> Option<&T> {
        self.back.last()
    }

    pub fn next(&self) -> Option<&T> {
        self.forward.last()
    }

    pub fn went_back(&mut self, left: T) {
        if self.back.pop().is_some() {
            self.forward.push(left);
        }
    }

    pub fn went_forward(&mut self, left: T) {
        if self.forward.pop().is_some() {
            self.back.push(left);
        }
    }
}
