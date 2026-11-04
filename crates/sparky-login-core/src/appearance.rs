use std::path::{Path, PathBuf};

use sparky_theme::locations::{Environment, color_files};
use sparky_theme::wallpapers::expand;

pub const FOLDER: &str = "/var/lib/sparky-login/appearance";
pub const COLORS: &str = "colors.toml";
pub const WALLPAPER: &str = "wallpaper";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sources {
    pub config: PathBuf,
    pub colors: PathBuf,
}

pub fn sources(home: &Path, config_home: Option<&str>) -> Sources {
    let home = home.to_string_lossy();
    let environment = Environment {
        home: &home,
        config_home,
        sparky_path: None,
    };
    let colors = color_files(&environment)
        .into_iter()
        .next()
        .unwrap_or_default();
    Sources {
        config: sparky_config::path(config_home, &home),
        colors,
    }
}

pub fn wallpaper(config: Option<&str>, home: &Path) -> Option<PathBuf> {
    let config = sparky_config::parse(config?).ok()?;
    let chosen = config.login_wallpaper()?.trim();
    if chosen.is_empty() {
        return None;
    }
    Some(expand(chosen, &home.to_string_lossy()))
}
