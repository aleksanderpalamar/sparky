use iced::{Size, Task};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sparky_theme::Skin;
use sparky_xdg::UserDirs;

use crate::adapters::archive::TarArchive;
use crate::adapters::clipboard::WlImageClipboard;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::icons::ThemeIcons;
use crate::adapters::local_disk::LocalDisk;
use crate::adapters::thumbnail::FileThumbnail;
use crate::adapters::xdg_open::XdgOpen;
use crate::app::message::Message;
use crate::app::state::{Files, Services, Setup};
use crate::domain::places::places;
use crate::view::style::{WINDOW_HEIGHT, WINDOW_WIDTH, rows_for};

pub fn boot() -> (Files, Task<Message>) {
    let dirs = UserDirs::from_env().unwrap_or_else(|| UserDirs::new("/", None));
    let home = dirs.home();
    let trash =
        HomeTrash::from_env().unwrap_or_else(|| HomeTrash::new(home.join(".local/share/Trash")));
    let setup = Setup {
        start: start_dir(env::args().nth(1), &home),
        places: places(&dirs)
            .into_iter()
            .filter(|place| place.path.is_dir())
            .collect(),
        home,
        skin: Skin::new(sparky_theme::current_palette()),
        rows: rows_for(WINDOW_HEIGHT),
    };
    let services = Services {
        disk: Box::new(LocalDisk::new(trash.clone())),
        opener: Box::new(XdgOpen),
        icons: Box::new(ThemeIcons::default()),
        volumes: Arc::new(sparky_storage::desktop::system()),
        archiver: Arc::new(TarArchive),
        thumbnails: Arc::new(FileThumbnail),
        trash: Arc::new(trash),
        image_clipboard: Box::new(WlImageClipboard),
    };
    let mut files = Files::new(setup, services);
    files.resize(Size::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let task = Task::batch([
        crate::app::volumes::refresh(&mut files),
        files.request_previews(),
    ]);
    (files, task)
}

pub fn start_dir(argument: Option<String>, home: &Path) -> PathBuf {
    argument
        .and_then(|path| std::path::absolute(path).ok())
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| home.to_path_buf())
}
