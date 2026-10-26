use std::path::{Path, PathBuf};

use sparky_xdg::icons::{IconTheme, configured_theme, icon_roots};

use crate::domain::indicator::icon_names;
use crate::domain::tray::TrayIcon;

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
            theme: IconTheme::new(roots, preferred),
            pixmaps,
        }
    }

    pub fn resolve(&self, name: &str, directory: Option<&str>) -> Option<TrayIcon> {
        if Path::new(name).is_absolute() {
            return Path::new(name).is_file().then(|| icon(PathBuf::from(name)));
        }
        let key = name
            .strip_suffix(".svg")
            .or_else(|| name.strip_suffix(".png"))
            .unwrap_or(name);
        let names = icon_names(key);
        let symbolic: Vec<_> = names
            .iter()
            .filter(|name| name.ends_with("-symbolic"))
            .map(String::as_str)
            .collect();
        if let Some(path) = self.theme.resolve(&symbolic) {
            return Some(icon(path));
        }
        if let Some(path) = directory.and_then(|directory| custom(Path::new(directory), name)) {
            return Some(icon(path));
        }
        let names: Vec<_> = names.iter().map(String::as_str).collect();
        self.theme
            .resolve(&names)
            .or_else(|| {
                self.pixmaps
                    .iter()
                    .find_map(|directory| custom(directory, name))
            })
            .map(icon)
    }
}

fn custom(directory: &Path, name: &str) -> Option<PathBuf> {
    if name.is_empty() || Path::new(name).components().count() != 1 || name == "." || name == ".." {
        return None;
    }
    [
        name.to_owned(),
        format!("{name}.svg"),
        format!("{name}.png"),
    ]
    .iter()
    .map(|name| directory.join(name))
    .find(|path| path.is_file())
}

fn icon(path: PathBuf) -> TrayIcon {
    let symbolic = path.extension().is_some_and(|extension| extension == "svg")
        && path.file_stem().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.ends_with("-symbolic") || name.ends_with("_mono")
        });
    match symbolic {
        true => TrayIcon::Symbolic(path),
        false => TrayIcon::File(path),
    }
}

#[cfg(test)]
#[path = "icons_tests.rs"]
mod tests;
