pub mod events;
pub mod keys;
pub mod loading;
pub mod message;
pub mod navigation;
pub mod state;
pub mod update;

#[cfg(test)]
mod keys_tests;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod viewer_tests;

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;
use sparky_theme::Skin;

use crate::adapters::decoder::ImageFile;
use crate::adapters::gallery::FolderGallery;
use crate::app::message::Message;
use crate::app::state::{Services, Viewer};
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot, update::update, view::view)
        .title(title)
        .subscription(events::subscription)
        .default_font(sparky_ui::style::FONT)
        .theme(|viewer: &Viewer| viewer.skin.theme())
        .window(sparky_ui::window::settings(
            style::APPLICATION_ID,
            style::WINDOW,
        ))
        .run()
}

fn boot() -> (Viewer, Task<Message>) {
    let services = Services {
        decoder: Arc::new(ImageFile),
        gallery: Box::new(FolderGallery),
    };
    let target = std::env::args_os().nth(1).map(PathBuf::from);
    Viewer::open(
        services,
        Skin::new(sparky_theme::current_palette()),
        target,
        style::WINDOW,
    )
}

pub fn title(viewer: &Viewer) -> String {
    viewer
        .album
        .as_ref()
        .and_then(|album| album.current().file_name())
        .map_or_else(
            || "Imagens".to_owned(),
            |name| format!("{} — Imagens", name.to_string_lossy()),
        )
}
