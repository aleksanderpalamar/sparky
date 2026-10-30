use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::{Message, Move};
use super::page::Page;
use crate::domain::table::Column;
use crate::ports::Stop;

pub fn message(key: &Key, modifiers: Modifiers) -> Option<Message> {
    match key {
        Key::Named(named) => named_message(*named, modifiers),
        Key::Character(text) => letter_message(text.as_str()),
        Key::Unidentified => None,
    }
}

fn named_message(named: Named, modifiers: Modifiers) -> Option<Message> {
    match named {
        Named::ArrowDown => Some(Message::Move(Move::Down)),
        Named::ArrowUp => Some(Message::Move(Move::Up)),
        Named::Home => Some(Message::Move(Move::First)),
        Named::End => Some(Message::Move(Move::Last)),
        Named::Tab => Some(Message::SwitchFocus),
        Named::Enter => Some(Message::Confirm),
        Named::Backspace => Some(Message::Open(Page::Dashboard)),
        Named::Delete if modifiers.shift() => Some(Message::Ask(Stop::Kill)),
        Named::Delete => Some(Message::Ask(Stop::Terminate)),
        _ => None,
    }
}

fn letter_message(text: &str) -> Option<Message> {
    match text {
        "j" => Some(Message::Move(Move::Down)),
        "k" => Some(Message::Move(Move::Up)),
        "n" => Some(Message::SortBy(Column::Name)),
        "c" => Some(Message::SortBy(Column::Cpu)),
        "m" => Some(Message::SortBy(Column::Memory)),
        "g" => Some(Message::SortBy(Column::Gpu)),
        "v" => Some(Message::SortBy(Column::Vram)),
        "d" => Some(Message::SortBy(Column::Disk)),
        "q" => Some(Message::Quit),
        digit => {
            let number = digit.parse::<usize>().ok()?;
            Page::nth(number.checked_sub(1)?).map(Message::Open)
        }
    }
}
