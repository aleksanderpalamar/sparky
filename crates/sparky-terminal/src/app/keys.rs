use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::sequences::{modifier_code, special};

const ESC: u8 = 0x1b;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    Normal,
    Application,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    PageUp,
    PageDown,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pressed {
    Send(Vec<u8>),
    Copy,
    Paste,
    Scroll(Scroll),
    Ignored,
}

pub fn pressed(
    key: &Key,
    modifiers: Modifiers,
    text: Option<&str>,
    latin: Option<char>,
    cursor: Cursor,
) -> Pressed {
    if let Some(found) = shortcut(key, modifiers, latin) {
        return found;
    }
    match key.as_ref() {
        Key::Named(named) => named_key(named, modifiers, cursor),
        Key::Character(character) => {
            character_key(character, &physical(character, latin), modifiers, text)
        }
        Key::Unidentified => Pressed::Ignored,
    }
}

fn shortcut(key: &Key, modifiers: Modifiers, latin: Option<char>) -> Option<Pressed> {
    let both = modifiers.control() && modifiers.shift();
    match key.as_ref() {
        Key::Character(letter) if both && physical(letter, latin).eq_ignore_ascii_case("c") => {
            Some(Pressed::Copy)
        }
        Key::Character(letter) if both && physical(letter, latin).eq_ignore_ascii_case("v") => {
            Some(Pressed::Paste)
        }
        Key::Named(Named::Insert) if modifiers.control() => Some(Pressed::Copy),
        Key::Named(Named::Insert) if modifiers.shift() => Some(Pressed::Paste),
        Key::Named(Named::PageUp) if modifiers.shift() => Some(Pressed::Scroll(Scroll::PageUp)),
        Key::Named(Named::PageDown) if modifiers.shift() => Some(Pressed::Scroll(Scroll::PageDown)),
        Key::Named(Named::Home) if modifiers.shift() => Some(Pressed::Scroll(Scroll::Top)),
        Key::Named(Named::End) if modifiers.shift() => Some(Pressed::Scroll(Scroll::Bottom)),
        _ => None,
    }
}

fn physical(letter: &str, latin: Option<char>) -> String {
    match (letter.is_ascii(), latin) {
        (false, Some(translated)) => translated.to_string(),
        _ => letter.to_owned(),
    }
}

fn named_key(named: Named, modifiers: Modifiers, cursor: Cursor) -> Pressed {
    let bytes = match named {
        Named::Enter => enter(modifiers),
        Named::Backspace => backspace(modifiers),
        Named::Tab if modifiers.shift() => b"\x1b[Z".to_vec(),
        Named::Tab => b"\t".to_vec(),
        Named::Escape => vec![ESC],
        Named::Space => return character_key(" ", " ", modifiers, Some(" ")),
        other => match special(other, modifier_code(modifiers), cursor) {
            Some(sequence) => sequence,
            None => return Pressed::Ignored,
        },
    };
    Pressed::Send(bytes)
}

fn enter(modifiers: Modifiers) -> Vec<u8> {
    match (modifiers.alt(), modifiers.shift()) {
        (true, true) => b"\x1b[13;4u".to_vec(),
        (false, true) => b"\x1b[13;2u".to_vec(),
        (true, false) => b"\x1b\r".to_vec(),
        (false, false) => b"\r".to_vec(),
    }
}

fn backspace(modifiers: Modifiers) -> Vec<u8> {
    match (modifiers.control(), modifiers.alt()) {
        (true, _) => vec![0x08],
        (false, true) => vec![ESC, 0x7f],
        (false, false) => vec![0x7f],
    }
}

fn character_key(key: &str, physical: &str, modifiers: Modifiers, text: Option<&str>) -> Pressed {
    let control = modifiers
        .control()
        .then(|| control_byte(physical))
        .flatten();
    let bytes = match control {
        Some(byte) => vec![byte],
        None => typed(key, text).as_bytes().to_vec(),
    };
    Pressed::Send(with_alt(bytes, modifiers))
}

fn typed<'a>(key: &'a str, text: Option<&'a str>) -> &'a str {
    text.filter(|text| !text.is_empty() && !text.chars().any(char::is_control))
        .unwrap_or(key)
}

fn with_alt(bytes: Vec<u8>, modifiers: Modifiers) -> Vec<u8> {
    match modifiers.alt() {
        true => [vec![ESC], bytes].concat(),
        false => bytes,
    }
}

fn control_byte(key: &str) -> Option<u8> {
    let mut characters = key.chars();
    let character = characters.next()?.to_ascii_lowercase();
    if characters.next().is_some() {
        return None;
    }
    match character {
        'a'..='z' => Some(character as u8 - b'a' + 1),
        '@' | ' ' | '2' => Some(0x00),
        '[' | '3' => Some(ESC),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '/' | '-' | '7' => Some(0x1f),
        '?' | '8' => Some(0x7f),
        _ => None,
    }
}
