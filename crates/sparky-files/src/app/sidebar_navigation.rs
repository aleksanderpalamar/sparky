use std::path::{Path, PathBuf};

use gio::prelude::*;

use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::sidebar_location::{SidebarLocation, SidebarTarget};

impl Files {
    pub fn open_sidebar(&mut self, page: SidebarLocation) {
        match self.services.sidebar.list(page) {
            Ok(items) => {
                self.sidebar_items = items;
                self.sidebar_page = Some(page);
                self.network_uri = None;
                self.sidebar_cursor = 0;
                self.mode = Mode::Browse;
            }
            Err(error) => self.fail(error),
        }
    }

    pub fn open_sidebar_item(&mut self, index: usize) {
        self.sidebar_cursor = index;
        let Some(target) = self
            .sidebar_items
            .get(index)
            .map(|item| item.target.clone())
        else {
            return;
        };
        match target {
            SidebarTarget::Local(path) => self.open_local_sidebar_item(path),
            SidebarTarget::Remote {
                uri,
                directory: true,
            } => self.open_network(&uri),
            SidebarTarget::Remote {
                uri,
                directory: false,
            } => {
                if let Err(error) = self.services.sidebar.open_remote(&uri) {
                    self.fail(error);
                }
            }
        }
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

    pub fn sync_favorite_moves(
        &self,
        clip: &Clip,
        names: &[String],
        destination: &Path,
    ) -> Result<(), String> {
        if !matches!(clip, Clip::Cut(_)) {
            return Ok(());
        }
        for (source, name) in clip.paths().iter().zip(names) {
            self.services
                .sidebar
                .relocated(source, &destination.join(name))?;
        }
        Ok(())
    }

    pub(super) fn open_network(&mut self, uri: &str) {
        match self.services.sidebar.network_children(uri) {
            Ok(items) => {
                self.sidebar_items = items;
                self.sidebar_page = Some(SidebarLocation::Network);
                self.network_uri = Some(uri.to_owned());
                self.sidebar_cursor = 0;
            }
            Err(error) => self.fail(error),
        }
    }

    pub(super) fn network_parent(&mut self) {
        let Some(uri) = self.network_uri.as_ref() else {
            return;
        };
        let file = gio::File::for_uri(uri);
        let Some(parent) = file.parent() else {
            self.open_sidebar(SidebarLocation::Network);
            return;
        };
        let parent_uri = parent.uri();
        if parent_uri.as_str() == "network:///" {
            self.open_sidebar(SidebarLocation::Network);
        } else {
            self.open_network(&parent_uri);
        }
    }
}
