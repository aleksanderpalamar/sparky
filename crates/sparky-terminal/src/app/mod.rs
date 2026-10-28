pub mod composer;
pub mod events;
pub mod input;
pub mod keys;
pub mod measure;
pub mod message;
pub mod paint;
pub mod pointer;
pub mod sequences;
pub mod signals;
pub mod state;
pub mod tab_keys;
pub mod tabs;
pub mod tabs_view;
pub mod update;
pub mod view;
pub mod workspace;

#[cfg(test)]
mod composer_tests;
#[cfg(test)]
mod interaction_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod pointer_tests;
#[cfg(test)]
mod pty_tabs_tests;
#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod tab_keys_tests;
#[cfg(test)]
mod view_tests;
#[cfg(test)]
mod workspace_tests;

use iced::{Size, Task, window};
use sparky_ui::style::FONT;

use crate::app::message::Message;
use crate::app::state::{HEIGHT, WIDTH};
use crate::app::workspace::Workspace;
use crate::domain::launch::Launch;

pub const APPLICATION_ID: &str = "sparky-terminal";

pub fn run(launch: Launch) -> iced::Result {
    let boot = move || {
        let cell = measure::cell();
        let terminal = Workspace::new(
            launch.clone(),
            sparky_theme::current_palette(),
            sparky_theme::current_terminal_palette(),
            cell,
        );
        (terminal, initial_scale())
    };
    iced::application(boot, update::update, tabs_view::view)
        .title(Workspace::title)
        .subscription(events::subscription)
        .theme(|terminal: &Workspace| terminal.skin.theme())
        .default_font(FONT)
        .window(sparky_ui::window::settings(
            APPLICATION_ID,
            Size::new(WIDTH, HEIGHT),
        ))
        .run()
}

fn initial_scale() -> Task<Message> {
    window::latest()
        .and_then(window::scale_factor)
        .map(Message::Rescaled)
}
