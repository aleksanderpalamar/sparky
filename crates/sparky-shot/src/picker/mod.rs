mod keys;
mod view;

use iced::{Element, Task, keyboard};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::surface::transparent;

use crate::domain::delay::Delay;
use crate::domain::mode::Mode;
use crate::picker::keys::{Command, command};
use crate::slot::Slot;

const NAMESPACE: &str = "sparky-shot-picker";
const SIZE: (u32, u32) = (320, 258);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    pub mode: Mode,
    pub delay: Delay,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Key(Command),
}

struct Picker {
    delay: Delay,
    skin: Skin,
    result: Slot<Choice>,
}

pub fn choose() -> Option<Choice> {
    let result = Slot::new();
    let shared = result.clone();
    let boot = move || Picker {
        delay: Delay::default(),
        skin: Skin::new(sparky_theme::current_palette()),
        result: shared.clone(),
    };
    let ran = iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view)
        .subscription(|_| iced::Subscription::batch([
            keyboard::listen().filter_map(key_message),
            sparky_ui::palette::subscription().map(Message::Palette),
        ]))
        .theme(|picker: &Picker| picker.skin.theme())
        .style(transparent)
        .settings(settings())
        .run();
    if let Err(error) = ran {
        eprintln!("sparky-shot: o seletor não abriu: {error}");
        return None;
    }
    result.take()
}

fn update(picker: &mut Picker, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => {
            picker.skin = Skin::new(palette);
            Task::none()
        }
        Message::Key(Command::Pick(mode)) => {
            picker.result.put(Choice { mode, delay: picker.delay });
            iced::exit()
        }
        Message::Key(Command::NextDelay) => {
            picker.delay = picker.delay.next();
            Task::none()
        }
        Message::Key(Command::Cancel) => iced::exit(),
        _ => Task::none(),
    }
}

fn view(picker: &Picker) -> Element<'_, Message> {
    view::view(picker.skin, picker.delay)
}

fn key_message(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed { key, .. } => command(&key).map(Message::Key),
        _ => None,
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some(SIZE),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        ..Settings::default()
    }
}
