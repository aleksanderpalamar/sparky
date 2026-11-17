mod config;
mod index;
mod rank;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub use config::{configured_theme, icon_roots, theme_from_settings};

const FORMATS: [&str; 2] = ["svg", "png"];
const LEGACY_SUFFIXES: [&str; 3] = [".png", ".svg", ".xpm"];

pub struct IconTheme {
    directories: Vec<PathBuf>,
}

impl IconTheme {
    pub fn new(roots: Vec<PathBuf>, preferred: &str) -> Self {
        Self::build(roots, preferred, None)
    }

    pub fn with_size(roots: Vec<PathBuf>, preferred: &str, pixels: u32) -> Self {
        Self::build(roots, preferred, Some(pixels))
    }

    fn build(roots: Vec<PathBuf>, preferred: &str, pixels: Option<u32>) -> Self {
        let mut directories = Vec::new();
        let mut visited = BTreeSet::new();
        for theme in [preferred, "hicolor", "Adwaita"] {
            collect(theme, &roots, pixels, &mut visited, &mut directories);
        }
        Self { directories }
    }

    pub fn resolve(&self, names: &[&str]) -> Option<PathBuf> {
        names
            .iter()
            .filter(|name| valid_name(name))
            .find_map(|name| {
                self.directories
                    .iter()
                    .find_map(|directory| find_in(directory, name))
            })
    }
}

pub fn find_in(directory: &Path, name: &str) -> Option<PathBuf> {
    let stem = LEGACY_SUFFIXES
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .filter(|stem| !stem.is_empty());
    let named = FORMATS
        .iter()
        .map(|format| directory.join(format!("{}.{format}", stem.unwrap_or(name))));
    let literal = stem.map(|_| directory.join(name));
    named.chain(literal).find(|path| path.is_file())
}

fn collect(
    theme: &str,
    roots: &[PathBuf],
    pixels: Option<u32>,
    visited: &mut BTreeSet<String>,
    directories: &mut Vec<PathBuf>,
) {
    if !valid_name(theme) || !visited.insert(theme.to_owned()) || visited.len() > 32 {
        return;
    }
    let Some(index) = roots
        .iter()
        .find_map(|root| index::read(&root.join(theme), pixels))
    else {
        return;
    };
    for directory in &index.directories {
        let existing = roots
            .iter()
            .map(|root| root.join(theme).join(directory))
            .filter(|path| path.is_dir());
        directories.extend(existing);
    }
    for parent in &index.inherits {
        collect(parent, roots, pixels, visited, directories);
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && Path::new(name).components().count() == 1
        && !name.contains('/')
}
