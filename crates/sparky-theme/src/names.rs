use std::fs;
use std::path::PathBuf;

use crate::locations;
use crate::palette::Palette;

const COLORS: &str = "colors.toml";

pub fn theme_name(palette: &Palette, themes: &[(String, String)]) -> Option<String> {
    themes
        .iter()
        .find(|(_, colors)| Palette::from_colors(colors) == *palette)
        .map(|(name, _)| name.clone())
}

pub fn installed_themes(roots: &[PathBuf]) -> Vec<(String, String)> {
    roots.iter().flat_map(themes_in).collect()
}

pub fn current_theme_name() -> Option<String> {
    let roots = locations::theme_roots_from_env();
    theme_name(&crate::current_palette(), &installed_themes(&roots))
}

fn themes_in(root: &PathBuf) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let colors = fs::read_to_string(entry.path().join(COLORS)).ok()?;
            Some((entry.file_name().to_string_lossy().into_owned(), colors))
        })
        .collect();
    found.sort();
    found
}
