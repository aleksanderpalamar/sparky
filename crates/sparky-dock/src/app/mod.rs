pub mod message;
pub mod state;
pub mod surfaces;
#[cfg(test)]
mod test_support;
pub mod update;
#[cfg(test)]
mod update_tests;
pub mod view;

use iced::{Subscription, Task, keyboard, window};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use sparky_theme::Skin;
use sparky_ui::style::{FONT, TEXT_SIZE};
use sparky_ui::surface::transparent;

use crate::adapters::{pinned_file::PinnedFile, start::Spawner, windows};
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

fn boot() -> (Dock, Task<Message>) {
    let skin = Skin::new(sparky_theme::current_palette());
    let dock = Dock::new(skin, Box::new(PinnedFile::from_env()), Box::new(Spawner));
    (dock, update::load_catalog())
}

fn namespace() -> String {
    surfaces::DOCK.to_owned()
}

fn subscription(dock: &Dock) -> Subscription<Message> {
    let picking = match dock.picker {
        Some(_) => Subscription::batch([
            keyboard::listen().filter_map(escape),
            window::open_events().map(Message::Opened),
        ]),
        None => Subscription::none(),
    };
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        Subscription::run(windows::changes).map(Message::Windows),
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
