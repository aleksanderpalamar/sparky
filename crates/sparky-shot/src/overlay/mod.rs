mod keys;
mod painter;

use iced::widget::canvas;
use iced::{Element, Length, Task, keyboard};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use sparky_theme::Skin;
use sparky_ui::surface::transparent;
use sparky_tiling::Rect;

use crate::domain::selection::Selection;
use crate::overlay::keys::{Command, command};
use crate::overlay::painter::Painter;
use crate::slot::Slot;

pub const NAMESPACE: &str = "sparky-shot";

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Pressed(i32, i32),
    Moved(i32, i32),
    Released(i32, i32),
    Key(Command),
}

struct Overlay {
    selection: Selection,
    skin: Skin,
    result: Slot<Rect>,
}

pub fn select(width: i32, height: i32) -> Option<Rect> {
    let result = Slot::new();
    let shared = result.clone();
    let boot = move || Overlay {
        selection: Selection::new(width, height),
        skin: Skin::new(sparky_theme::current_palette()),
        result: shared.clone(),
    };
    let ran = iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view)
        .subscription(|_| keyboard::listen().filter_map(key_message))
        .style(transparent)
        .settings(settings())
        .run();
    if let Err(error) = ran {
        eprintln!("sparky-shot: a seleção não abriu: {error}");
        return None;
    }
    result.take()
}

fn update(overlay: &mut Overlay, message: Message) -> Task<Message> {
    let selection = &mut overlay.selection;
    match message {
        Message::Pressed(x, y) => selection.press(x, y),
        Message::Moved(x, y) => selection.drag(x, y),
        Message::Released(x, y) => selection.release(x, y),
        Message::Key(Command::Nudge(dx, dy)) => selection.nudge(dx, dy),
        Message::Key(Command::Grow(dw, dh)) => selection.grow(dw, dh),
        Message::Key(Command::Whole) => selection.whole(),
        Message::Key(Command::Confirm) => {
            overlay.result.put(overlay.selection.rect());
            return iced::exit();
        }
        Message::Key(Command::Cancel) => return iced::exit(),
        _ => {}
    }
    Task::none()
}

fn view(overlay: &Overlay) -> Element<'_, Message> {
    let painter = Painter {
        selection: overlay.selection.rect(),
        skin: overlay.skin,
    };
    canvas(painter).width(Length::Fill).height(Length::Fill).into()
}

fn key_message(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed { key, modifiers, .. } => command(&key, modifiers).map(Message::Key),
        _ => None,
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: -1,
            size: None,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        ..Settings::default()
    }
}
