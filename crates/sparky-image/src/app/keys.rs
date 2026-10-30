use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::domain::album::{Edge, Step};
use crate::domain::turn::Spin;
use crate::domain::viewport::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Arrow(Direction),
    Step(Step),
    Jump(Edge),
    ZoomIn,
    ZoomOut,
    Fit,
    ActualSize,
    Rotate(Spin),
    Quit,
}

pub fn command(key: &Key, modifiers: Modifiers) -> Option<Command> {
    match key {
        Key::Named(named) => named_command(*named),
        Key::Character(text) => letter_command(text.as_str(), modifiers),
        Key::Unidentified => None,
    }
}

fn named_command(named: Named) -> Option<Command> {
    match named {
        Named::ArrowLeft => Some(Command::Arrow(Direction::Left)),
        Named::ArrowRight => Some(Command::Arrow(Direction::Right)),
        Named::ArrowUp => Some(Command::Arrow(Direction::Up)),
        Named::ArrowDown => Some(Command::Arrow(Direction::Down)),
        Named::Space | Named::PageDown => Some(Command::Step(Step::Next)),
        Named::Backspace | Named::PageUp => Some(Command::Step(Step::Previous)),
        Named::Home => Some(Command::Jump(Edge::First)),
        Named::End => Some(Command::Jump(Edge::Last)),
        _ => None,
    }
}

fn letter_command(text: &str, modifiers: Modifiers) -> Option<Command> {
    match text {
        "h" => Some(Command::Arrow(Direction::Left)),
        "j" => Some(Command::Arrow(Direction::Down)),
        "k" => Some(Command::Arrow(Direction::Up)),
        "l" => Some(Command::Arrow(Direction::Right)),
        "+" | "=" => Some(Command::ZoomIn),
        "-" => Some(Command::ZoomOut),
        "0" => Some(Command::Fit),
        "1" => Some(Command::ActualSize),
        "r" if modifiers.shift() => Some(Command::Rotate(Spin::CounterClockwise)),
        "r" => Some(Command::Rotate(Spin::Clockwise)),
        "R" => Some(Command::Rotate(Spin::CounterClockwise)),
        "q" => Some(Command::Quit),
        _ => None,
    }
}
