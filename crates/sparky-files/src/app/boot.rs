use std::env;
use std::path::{Path, PathBuf};

use sparky_theme::Skin;
use sparky_xdg::UserDirs;

use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::adapters::xdg_open::XdgOpen;
use crate::app::state::{Files, Services, Setup};
use crate::domain::places::places;
use crate::view::style::{WINDOW_HEIGHT, rows_for};

pub fn boot() -> Files {
    let dirs = UserDirs::from_env().unwrap_or_else(|| UserDirs::new("/", None));
    let home = dirs.home();
    let trash = HomeTrash::from_env().unwrap_or_else(|| HomeTrash::new(home.join(".local/share/Trash")));
    let setup = Setup {
        start: start_dir(env::args().nth(1), &home),
        places: places(&dirs).into_iter().filter(|place| place.path.is_dir()).collect(),
        home,
        skin: Skin::new(sparky_theme::current_palette()),
        rows: rows_for(WINDOW_HEIGHT),
    };
    let services = Services {
        disk: Box::new(LocalDisk::new(trash)),
        opener: Box::new(XdgOpen),
    };
    Files::new(setup, services)
}

pub fn start_dir(argument: Option<String>, home: &Path) -> PathBuf {
    argument
        .and_then(|path| std::path::absolute(path).ok())
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| home.to_path_buf())
}
