use std::path::{Path, PathBuf};

use sparky_ui::viewport::Viewport;

use crate::app::state::{Files, Mode};
use crate::domain::entry::Kind;
use crate::domain::listing::Listing;

impl Files {
    pub fn go(&mut self, path: PathBuf) -> bool {
        match self.services.disk.list(&path) {
            Ok(entries) => {
                self.listing = Listing::new(entries, self.listing.hidden());
                self.viewport = Viewport::new(self.viewport.rows());
                self.stamp = self.services.disk.stamp(&path);
                self.location = path;
                self.mode = Mode::Browse;
                true
            }
            Err(error) => {
                self.fail(format!("{}: {error}", self.shown(&path)));
                false
            }
        }
    }

    pub fn open_selected(&mut self) {
        if self.in_trash() {
            self.inform("Restaure o item para abri-lo".to_owned());
            return;
        }
        let Some(entry) = self.listing.selected() else {
            return;
        };
        let path = self.location.join(&entry.name);
        match entry.kind {
            Kind::Directory => {
                self.go(path);
            }
            Kind::File => self.launch(&path),
        }
    }

    pub fn parent(&mut self) {
        if self.in_trash() {
            self.home();
            return;
        }
        let Some(parent) = self.location.parent().map(Path::to_path_buf) else {
            return;
        };
        let child = self
            .location
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        if self.go(parent)
            && let Some(child) = child
        {
            self.listing.select_name(&child);
        }
    }

    pub fn home(&mut self) {
        self.go(self.home.clone());
    }

    pub fn place(&mut self, index: usize) {
        if let Some(place) = self.places.get(index) {
            self.go(place.path.clone());
        }
    }

    pub fn refresh(&mut self) {
        match self.services.disk.list(&self.location) {
            Ok(entries) => {
                self.listing.replace(entries);
                self.stamp = self.services.disk.stamp(&self.location);
            }
            Err(error) => self.fail(format!("{}: {error}", self.shown(&self.location))),
        }
        self.follow();
    }

    pub fn refresh_if_changed(&mut self) {
        if self.services.disk.stamp(&self.location) != self.stamp {
            self.refresh();
        }
    }

    fn launch(&mut self, path: &Path) {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        match self.services.opener.open(path) {
            Ok(()) => self.inform(format!("Abrindo {name}")),
            Err(error) => self.fail(format!("{name}: {error}")),
        }
    }
}
