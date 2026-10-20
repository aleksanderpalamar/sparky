use std::path::PathBuf;

use sparky_theme::Skin;

use crate::adapters::config_file::ConfigFile;
use crate::adapters::sparky_ipc::SparkyIpc;
use crate::adapters::system;
use crate::app::state::{Context, Services, Settings};
use crate::view::style::{WINDOW_HEIGHT, rows_for};

pub fn boot() -> Settings {
    let services = Services {
        store: Box::new(ConfigFile::from_env()),
        compositor: Box::new(SparkyIpc),
    };
    let context = Context {
        home: std::env::var("HOME").map(PathBuf::from).unwrap_or_default(),
        about: system::about(),
        keymaps: system::keymaps(),
        skin: Skin::new(sparky_theme::current_palette()),
        rows: rows_for(WINDOW_HEIGHT),
    };
    Settings::new(services, context)
}
