use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;

use crate::app::keys::Cursor;

const PLAIN: u8 = 1;

pub fn modifier_code(modifiers: Modifiers) -> u8 {
    PLAIN
        + u8::from(modifiers.shift())
        + 2 * u8::from(modifiers.alt())
        + 4 * u8::from(modifiers.control())
}

pub fn special(named: Named, code: u8, cursor: Cursor) -> Option<Vec<u8>> {
    if let Some(letter) = cursor_letter(named) {
        return Some(cursor_sequence(letter, code, cursor));
    }
    if let Some(number) = tilde_number(named) {
        return Some(tilde(number, code));
    }
    function_letter(named).map(|letter| function(letter, code))
}

fn cursor_letter(named: Named) -> Option<char> {
    match named {
        Named::ArrowUp => Some('A'),
        Named::ArrowDown => Some('B'),
        Named::ArrowRight => Some('C'),
        Named::ArrowLeft => Some('D'),
        Named::Home => Some('H'),
        Named::End => Some('F'),
        _ => None,
    }
}

fn tilde_number(named: Named) -> Option<u8> {
    match named {
        Named::Insert => Some(2),
        Named::Delete => Some(3),
        Named::PageUp => Some(5),
        Named::PageDown => Some(6),
        Named::F5 => Some(15),
        Named::F6 => Some(17),
        Named::F7 => Some(18),
        Named::F8 => Some(19),
        Named::F9 => Some(20),
        Named::F10 => Some(21),
        Named::F11 => Some(23),
        Named::F12 => Some(24),
        _ => None,
    }
}

fn function_letter(named: Named) -> Option<char> {
    match named {
        Named::F1 => Some('P'),
        Named::F2 => Some('Q'),
        Named::F3 => Some('R'),
        Named::F4 => Some('S'),
        _ => None,
    }
}

fn cursor_sequence(letter: char, code: u8, cursor: Cursor) -> Vec<u8> {
    match (code, cursor) {
        (PLAIN, Cursor::Application) => format!("\x1bO{letter}").into_bytes(),
        (PLAIN, Cursor::Normal) => format!("\x1b[{letter}").into_bytes(),
        (modified, _) => format!("\x1b[1;{modified}{letter}").into_bytes(),
    }
}

fn tilde(number: u8, code: u8) -> Vec<u8> {
    match code {
        PLAIN => format!("\x1b[{number}~").into_bytes(),
        modified => format!("\x1b[{number};{modified}~").into_bytes(),
    }
}

fn function(letter: char, code: u8) -> Vec<u8> {
    match code {
        PLAIN => format!("\x1bO{letter}").into_bytes(),
        modified => format!("\x1b[1;{modified}{letter}").into_bytes(),
    }
}
