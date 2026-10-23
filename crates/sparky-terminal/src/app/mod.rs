pub mod composer;
pub mod events;
pub mod input;
pub mod keys;
pub mod measure;
pub mod message;
pub mod paint;
pub mod sequences;
pub mod state;
pub mod update;
pub mod view;

#[cfg(test)]
mod composer_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod view_tests;

use iced::{Size, Task, window};
use sparky_ui::style::FONT;

use crate::app::message::Message;
use crate::app::state::{HEIGHT, Terminal, WIDTH};
use crate::domain::launch::Launch;

pub const APPLICATION_ID: &str = "sparky-terminal";

pub fn run(launch: Launch) -> iced::Result {
    let boot = move || {
        let cell = measure::cell();
        let terminal = Terminal::new(
            launch.clone(),
            sparky_theme::current_palette(),
            sparky_theme::current_terminal_palette(),
            cell,
        );
        (terminal, initial_scale())
    };
    iced::application(boot, update::update, view::view)
        .title(|terminal: &Terminal| terminal.title.clone())
        .subscription(events::subscription)
        .theme(|terminal: &Terminal| terminal.skin.theme())
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
