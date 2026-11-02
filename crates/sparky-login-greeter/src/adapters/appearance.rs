use std::fs;
use std::path::{Path, PathBuf};

use sparky_login_core::appearance::{COLORS, WALLPAPER};
use sparky_theme::Palette;
use sparky_theme::locations::color_files_from_env;

pub fn palette(folder: &Path) -> Palette {
    palette_from(folder, color_files_from_env())
}

pub fn palette_from(folder: &Path, defaults: Vec<PathBuf>) -> Palette {
    let candidates: Vec<PathBuf> = std::iter::once(folder.join(COLORS))
        .chain(defaults)
        .collect();
    Palette::load(|path| fs::read_to_string(path).ok(), &candidates)
}

pub fn wallpaper(folder: &Path) -> Option<PathBuf> {
    let chosen = folder.join(WALLPAPER);
    chosen.is_file().then_some(chosen)
}
