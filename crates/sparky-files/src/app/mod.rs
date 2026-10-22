pub mod boot;
pub mod editing;
pub mod events;
pub mod keys;
pub mod message;
pub mod navigation;
pub mod presentation;
pub mod volumes;
pub mod state;
pub mod update;

#[cfg(test)]
mod files_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod test_sources;
#[cfg(test)]
mod presentation_tests;
#[cfg(test)]
mod volumes_tests;

use iced::Size;

use crate::app::state::Files;
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot::boot, update::update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .default_font(sparky_ui::style::FONT)
        .theme(|files: &Files| files.skin.theme())
        .window(sparky_ui::window::settings(style::APPLICATION_ID, Size::new(style::WINDOW_WIDTH, style::WINDOW_HEIGHT)))
        .run()
}

