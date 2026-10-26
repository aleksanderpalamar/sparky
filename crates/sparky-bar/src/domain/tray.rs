use std::path::PathBuf;

use super::indicator::{Role, Status};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayIcon {
    File(PathBuf),
    Symbolic(PathBuf),
    Pixels {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayRequest {
    Activate {
        address: String,
    },
    Select {
        address: String,
        menu_path: String,
        entry: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryAction {
    Activate,
    Menu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub separator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub address: String,
    pub title: String,
    pub icon: TrayIcon,
    pub role: Role,
    pub status: Status,
    pub primary_action: PrimaryAction,
    pub menu_path: Option<String>,
    pub menu: Vec<MenuEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayChange {
    Added(TrayItem),
    Icon {
        address: String,
        icon: TrayIcon,
    },
    Title {
        address: String,
        title: String,
    },
    Menu {
        address: String,
        menu: Vec<MenuEntry>,
    },
    Status {
        address: String,
        status: Status,
    },
    Removed(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tray {
    items: Vec<TrayItem>,
}

impl Tray {
    pub fn items(&self) -> &[TrayItem] {
        &self.items
    }

    pub fn item(&self, address: &str) -> Option<&TrayItem> {
        self.items.iter().find(|item| item.address == address)
    }

    pub fn apply(&mut self, change: TrayChange) {
        match change {
            TrayChange::Added(item) => self.add(item),
            TrayChange::Icon { address, icon } => self.edit(&address, |item| item.icon = icon),
            TrayChange::Title { address, title } => self.edit(&address, |item| item.title = title),
            TrayChange::Menu { address, menu } => self.edit(&address, |item| item.menu = menu),
            TrayChange::Status { address, status } => {
                self.edit(&address, |item| item.status = status)
            }
            TrayChange::Removed(address) => self.items.retain(|item| item.address != address),
        }
    }

    fn add(&mut self, item: TrayItem) {
        match self
            .items
            .iter_mut()
            .find(|known| known.address == item.address)
        {
            Some(known) => *known = item,
            None => self.items.push(item),
        }
    }

    fn edit(&mut self, address: &str, change: impl FnOnce(&mut TrayItem)) {
        if let Some(item) = self.items.iter_mut().find(|item| item.address == address) {
            change(item);
        }
    }
}

#[cfg(test)]
#[path = "tray_tests.rs"]
mod tests;
