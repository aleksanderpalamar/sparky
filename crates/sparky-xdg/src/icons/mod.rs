mod config;
mod index;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub use config::{configured_theme, icon_roots, theme_from_settings};

pub struct IconTheme {
    directories: Vec<PathBuf>,
}

impl IconTheme {
    pub fn new(roots: Vec<PathBuf>, preferred: &str) -> Self {
        let mut directories = Vec::new();
        let mut visited = BTreeSet::new();
        for theme in [preferred, "hicolor", "Adwaita"] {
            collect(theme, &roots, &mut visited, &mut directories);
        }
        Self { directories }
    }

    pub fn resolve(&self, names: &[&str]) -> Option<PathBuf> {
        names.iter().filter(|name| valid_name(name)).find_map(|name| {
            self.directories.iter().find_map(|directory| {
                ["svg", "png"].iter().map(|extension| directory.join(format!("{name}.{extension}"))).find(|path| path.is_file())
            })
        })
    }
}

fn collect(theme: &str, roots: &[PathBuf], visited: &mut BTreeSet<String>, directories: &mut Vec<PathBuf>) {
    if !valid_name(theme) || !visited.insert(theme.to_owned()) || visited.len() > 32 {
        return;
    }
    let mut inherited = Vec::new();
    for root in roots {
        let directory = root.join(theme);
        let theme = index::read(&directory);
        directories.extend(theme.directories);
        inherited.extend(theme.inherits);
    }
    for parent in inherited {
        collect(&parent, roots, visited, directories);
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && Path::new(name).components().count() == 1 && !name.contains('/')
}
