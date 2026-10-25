use super::keys::physical;
use crate::domain::tabs::TabAction;
use iced::keyboard::{Key, Modifiers, key::Named};

pub fn shortcut(key: &Key, modifiers: Modifiers, latin: Option<char>) -> Option<TabAction> {
    if modifiers != Modifiers::CTRL | Modifiers::SHIFT {
        return None;
    }
    match key.as_ref() {
        Key::Character(letter) if physical(letter, latin).eq_ignore_ascii_case("t") => {
            Some(TabAction::New)
        }
        Key::Character(letter) if physical(letter, latin).eq_ignore_ascii_case("q") => {
            Some(TabAction::Close)
        }
        Key::Named(Named::ArrowRight) => Some(TabAction::Cycle(1)),
        Key::Named(Named::ArrowLeft) => Some(TabAction::Cycle(-1)),
        _ => None,
    }
}
