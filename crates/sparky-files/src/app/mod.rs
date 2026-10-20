pub mod boot;
pub mod editing;
pub mod events;
pub mod keys;
pub mod message;
pub mod navigation;
pub mod prompt;
pub mod state;
pub mod update;

#[cfg(test)]
mod files_tests;
#[cfg(test)]
mod keys_tests;

use iced::{Size, window};

use crate::app::state::Files;
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot::boot, update::update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .theme(|files: &Files| files.skin.theme())
        .window(window_settings())
        .run()
}

fn window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(style::WINDOW_WIDTH, style::WINDOW_HEIGHT),
        platform_specific: window::settings::PlatformSpecific {
            application_id: style::APPLICATION_ID.to_owned(),
            ..window::settings::PlatformSpecific::default()
        },
        ..window::Settings::default()
    }
}
