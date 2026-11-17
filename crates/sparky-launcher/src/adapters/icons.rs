use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sparky_xdg::icons::{IconTheme, configured_theme, find_in, icon_roots};

use crate::adapters::raster::raster;
use crate::domain::app::App;
use crate::domain::icon::{ICON_PIXELS, Icon, Picture};

const PIXELS: u32 = 32;

pub struct Icons {
    theme: IconTheme,
    pixmaps: Vec<PathBuf>,
}

impl Icons {
    pub fn current() -> Self {
        Self::new(icon_roots(), &configured_theme())
    }

    pub fn new(roots: Vec<PathBuf>, preferred: &str) -> Self {
        let pixmaps = roots
            .iter()
            .filter(|root| root.file_name().is_some_and(|name| name == "icons"))
            .filter_map(|root| root.parent().map(|parent| parent.join("pixmaps")))
            .collect();
        Self {
            theme: IconTheme::with_size(roots, preferred, PIXELS),
            pixmaps,
        }
    }

    pub fn resolve(&self, apps: &[App]) -> HashMap<String, Picture> {
        apps.iter()
            .filter_map(|app| {
                let path = self.path(app.icon.as_ref()?)?;
                Some((app.id.clone(), picture(path)?))
            })
            .collect()
    }

    pub fn path(&self, icon: &Icon) -> Option<PathBuf> {
        match icon {
            Icon::Symbolic(names) => self.theme.resolve(names),
            Icon::Named(name) if Path::new(name).is_absolute() => {
                Path::new(name).is_file().then(|| PathBuf::from(name))
            }
            Icon::Named(name) => self.theme.resolve(&[name]).or_else(|| self.pixmap(name)),
        }
    }

    fn pixmap(&self, name: &str) -> Option<PathBuf> {
        if Path::new(name).components().count() != 1 || name == "." || name == ".." {
            return None;
        }
        self.pixmaps
            .iter()
            .find_map(|directory| find_in(directory, name))
    }
}

fn picture(path: PathBuf) -> Option<Picture> {
    match path.extension().is_some_and(|extension| extension == "svg") {
        true => Some(Picture::Vector(path)),
        false => raster(&path, ICON_PIXELS).map(Picture::Raster),
    }
}
