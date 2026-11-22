mod animate;
mod contact;
#[cfg(test)]
mod contact_tests;
#[cfg(test)]
mod dock_tests;
pub mod message;
mod pointer;
#[cfg(test)]
mod pointer_tests;
mod restore;
pub mod state;
mod surface;
pub mod surfaces;
#[cfg(test)]
mod test_support;
pub mod update;
#[cfg(test)]
mod update_tests;
pub mod view;

use std::time::Duration;

use iced::{Subscription, Task, keyboard, time, window};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use sparky_theme::Skin;
use sparky_ui::style::{FONT, TEXT_SIZE};
use sparky_ui::surface::transparent;

use crate::adapters::{pinned_file::PinnedFile, preferences, start::Spawner, windows};
use message::Message;
use state::Dock;

pub fn run() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::daemon(boot, namespace, update::update, view::view)
        .subscription(subscription)
        .theme(|dock: &Dock, _| dock.skin.theme())
        .style(transparent)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                start_mode: StartMode::Background,
                ..LayerShellSettings::default()
            },
            default_font: FONT,
            default_text_size: TEXT_SIZE.into(),
            antialiasing: true,
            ..Settings::default()
        })
        .run()
}

const FRAME: Duration = Duration::from_millis(16);

fn boot() -> (Dock, Task<Message>) {
    let skin = Skin::new(sparky_theme::current_palette());
    let dock = Dock::new(skin, Box::new(PinnedFile::from_env()), Box::new(Spawner));
    (
        dock,
        Task::batch([update::load_catalog(), update::measure()]),
    )
}

fn namespace() -> String {
    surfaces::DOCK.to_owned()
}

fn subscription(dock: &Dock) -> Subscription<Message> {
    let picking = match dock.picker {
        Some(_) => keyboard::listen().filter_map(escape),
        None => Subscription::none(),
    };
    let frames = match animate::animating(dock) {
        true => time::every(FRAME).map(Message::Frame),
        false => Subscription::none(),
    };
    Subscription::batch([
        frames,
        sparky_ui::palette::subscription().map(Message::Palette),
        Subscription::run(windows::changes).map(Message::Windows),
        window::open_events().map(Message::Opened),
        iced::event::listen_with(contact::surface),
        Subscription::run(preferences::changes).map(Message::Preferences),
        picking,
    ])
}

fn escape(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        } => Some(Message::Cancel),
        _ => None,
    }
}
