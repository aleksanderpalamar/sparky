use std::path::PathBuf;

use gio::prelude::*;
use iced::Task;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation, SidebarTarget};

impl Files {
    pub fn open_sidebar(&mut self, page: SidebarLocation) {
        match self.services.sidebar.list(page) {
            Ok(items) => {
                self.sidebar_items = items;
                self.sidebar_page = Some(page);
                self.network_uri = None;
                self.network_pending = None;
                self.sidebar_cursor = 0;
                self.mode = Mode::Browse;
            }
            Err(error) => self.fail(error),
        }
    }

    pub fn open_sidebar_item(&mut self, index: usize) -> Task<Message> {
        self.sidebar_cursor = index;
        let Some(target) = self
            .sidebar_items
            .get(index)
            .map(|item| item.target.clone())
        else {
            return Task::none();
        };
        match target {
            SidebarTarget::Local(path) => self.open_local_sidebar_item(path),
            SidebarTarget::Remote {
                uri,
                directory: true,
            } => return self.load_network(uri),
            SidebarTarget::Remote {
                uri,
                directory: false,
            } => {
                if let Err(error) = self.services.sidebar.open_remote(&uri) {
                    self.fail(error);
                }
            }
        }
        Task::none()
    }

    fn open_local_sidebar_item(&mut self, path: PathBuf) {
        if !path.exists() {
            self.refresh();
            self.fail(format!("{}: arquivo indisponível", path.display()));
            return;
        }
        if path.is_dir() {
            self.go(path);
        } else {
            self.launch(&path);
        }
    }

    pub fn toggle_favorite(&mut self) {
        let Some(entry) = self.listing.selected() else {
            return;
        };
        let path = self.location.join(&entry.name);
        let enabled = !self.services.sidebar.is_favorite(&path);
        match self.services.sidebar.favorite(&path, enabled) {
            Ok(()) => self.inform(crate::i18n::text(if enabled {
                "favorite-added"
            } else {
                "favorite-removed"
            })),
            Err(error) => self.fail(error),
        }
    }

    pub fn remove_favorite(&mut self, index: usize) {
        let Some(SidebarTarget::Local(path)) = self
            .sidebar_items
            .get(index)
            .map(|item| item.target.clone())
        else {
            return;
        };
        match self.services.sidebar.favorite(&path, false) {
            Ok(()) => self.open_sidebar(SidebarLocation::Favorites),
            Err(error) => self.fail(error),
        }
    }

    pub(super) fn load_network(&mut self, uri: String) -> Task<Message> {
        self.network_pending = Some(uri.clone());
        let source = self.services.sidebar.clone();
        Task::perform(
            async move {
                let target = uri.clone();
                let result = tokio::task::spawn_blocking(move || source.network_children(&target))
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result);
                (uri, result)
            },
            |(uri, result)| Message::NetworkLoaded(uri, result),
        )
    }

    pub(super) fn network_loaded(&mut self, uri: &str, result: Result<Vec<SidebarItem>, String>) {
        if self.network_pending.as_deref() != Some(uri) {
            return;
        }
        self.network_pending = None;
        match result {
            Ok(items) => {
                self.sidebar_items = items;
                self.sidebar_page = Some(SidebarLocation::Network);
                self.network_uri = Some(uri.to_owned());
                self.sidebar_cursor = 0;
            }
            Err(error) => self.fail(error),
        }
    }

    pub(super) fn network_parent(&mut self) -> Task<Message> {
        let Some(uri) = self.network_uri.as_ref() else {
            return Task::none();
        };
        let file = gio::File::for_uri(uri);
        let Some(parent) = file.parent() else {
            self.open_sidebar(SidebarLocation::Network);
            return Task::none();
        };
        let parent_uri = parent.uri();
        if parent_uri.as_str() == "network:///" {
            self.open_sidebar(SidebarLocation::Network);
            return Task::none();
        }
        self.load_network(parent_uri.to_string())
    }
}
