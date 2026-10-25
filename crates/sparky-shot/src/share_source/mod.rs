mod model;
mod view;

#[cfg(test)]
mod tests;

use crate::slot::Slot;
use iced::{Task, keyboard};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    to_layer_message,
};
use model::Selection;
use sparky_theme::Skin;
use std::io::{self, Read, Write};

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Search(String),
    Step(isize),
    Pick(usize),
    Accept,
    Cancel,
}

struct Picker {
    selection: Selection,
    skin: Skin,
    result: Slot<String>,
}

pub fn run() -> io::Result<()> {
    let mut input = String::new();
    io::stdin()
        .take(1024 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "lista de fontes muito grande",
        ));
    }
    if input.lines().all(str::is_empty) {
        return Ok(());
    }
    let Some(selected) = choose(input) else {
        return Ok(());
    };
    writeln!(io::stdout().lock(), "{selected}")
}

fn choose(input: String) -> Option<String> {
    let result = Slot::new();
    let shared = result.clone();
    let boot = move || Picker {
        selection: Selection::new(&input),
        skin: Skin::new(sparky_theme::current_palette()),
        result: shared.clone(),
    };
    let ran =
        iced_layershell::application(boot, || "sparky-share-source".into(), update, view::view)
            .subscription(|_| {
                iced::Subscription::batch([
                    keyboard::listen().filter_map(key),
                    sparky_ui::palette::subscription().map(Message::Palette),
                ])
            })
            .theme(|picker: &Picker| picker.skin.theme())
            .style(sparky_ui::surface::transparent)
            .settings(settings())
            .run();
    if let Err(error) = ran {
        eprintln!("sparky-shot: o seletor de compartilhamento não abriu: {error}");
        return None;
    }
    result.take()
}

fn update(picker: &mut Picker, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => picker.skin = Skin::new(palette),
        Message::Search(query) => {
            picker.selection.search(query);
            return iced::widget::operation::snap_to(
                view::list_id(),
                iced::widget::scrollable::RelativeOffset::START,
            );
        }
        Message::Step(delta) => {
            picker.selection.step(delta);
            let count = picker.selection.visible().len().saturating_sub(1).max(1);
            let y = picker.selection.selected as f32 / count as f32;
            return iced::widget::operation::snap_to(
                view::list_id(),
                iced::widget::scrollable::RelativeOffset { x: 0.0, y },
            );
        }
        Message::Pick(index) => {
            let Some(source) = picker.selection.sources.get(index) else {
                return Task::none();
            };
            picker.result.put(source.raw.clone());
            return iced::exit();
        }
        Message::Accept => {
            let Some(source) = picker.selection.chosen() else {
                return Task::none();
            };
            picker.result.put(source.raw.clone());
            return iced::exit();
        }
        Message::Cancel => return iced::exit(),
        _ => {}
    }
    Task::none()
}

fn key(event: keyboard::Event) -> Option<Message> {
    use keyboard::{Key, key::Named};
    let keyboard::Event::KeyPressed {
        key: Key::Named(key),
        ..
    } = event
    else {
        return None;
    };
    match key {
        Named::ArrowUp => Some(Message::Step(-1)),
        Named::ArrowDown => Some(Message::Step(1)),
        Named::Enter => Some(Message::Accept),
        Named::Escape => Some(Message::Cancel),
        _ => None,
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((640, 480)),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: sparky_ui::style::FONT,
        ..Settings::default()
    }
}
