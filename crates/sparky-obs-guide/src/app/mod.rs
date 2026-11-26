use iced::event::{self, Event, Status};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::{Subscription, Task, window};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::press::{Press, press};
use sparky_ui::surface::transparent;

use crate::preference::{location, silence};
use crate::view;

const NAMESPACE: &str = "sparky-obs-guide";
const WIDTH: u32 = 540;
const HEIGHT: u32 = 340;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Understood,
    Never,
}

pub struct Window {
    pub skin: Skin,
}

pub fn run() -> Result<(), iced_layershell::Error> {
    let boot = || {
        let window = Window {
            skin: Skin::new(sparky_theme::current_palette()),
        };
        (window, Task::none())
    };
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view::view)
        .subscription(|_| {
            Subscription::batch([
                event::listen_with(keyboard_message),
                sparky_ui::palette::subscription().map(Message::Palette),
            ])
        })
        .theme(|window: &Window| window.skin.theme())
        .style(transparent)
        .settings(settings())
        .run()
}

fn update(window: &mut Window, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => window.skin = Skin::new(palette),
        Message::Understood => return iced::exit(),
        Message::Never => {
            if let Err(error) = silence(&location()) {
                eprintln!("sparky-obs-guide: preferência não gravada: {error}");
            }
            return iced::exit();
        }
        _ => {}
    }
    Task::none()
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Understood),
        Press::Key(Key::Named(Named::Enter), _) => Some(Message::Understood),
        Press::Key(..) => None,
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((WIDTH, HEIGHT)),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: sparky_ui::style::FONT,
        ..Settings::default()
    }
}
