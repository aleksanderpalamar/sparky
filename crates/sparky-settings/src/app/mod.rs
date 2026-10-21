pub mod actions;
pub mod boot;
pub mod change;
pub mod events;
pub mod keys;
pub mod message;
pub mod state;
pub mod system;
pub mod update;

#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod datetime_tests;
#[cfg(test)]
mod region_tests;
#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod test_support;

use iced::Size;

use crate::app::state::Settings;
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot::boot, update::update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .theme(|settings: &Settings| settings.context.skin.theme())
        .window(sparky_ui::window::settings(style::APPLICATION_ID, Size::new(style::WINDOW_WIDTH, style::WINDOW_HEIGHT)))
        .run()
}
