use std::str::Chars;

use sparky_theme::Rgb;
use unicode_width::UnicodeWidthStr;

const ESCAPE: char = '\x1b';
const BELL: char = '\x07';

pub(crate) fn paint(text: &str, color: Rgb) -> String {
    format!(
        "{ESCAPE}[38;2;{};{};{}m{text}{ESCAPE}[0m",
        color.red, color.green, color.blue
    )
}

pub(crate) fn width(text: &str) -> usize {
    strip(text).width()
}

pub(crate) fn strip(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(character) = chars.next() {
        match character {
            ESCAPE => skip_sequence(&mut chars),
            c if c.is_control() && c.is_whitespace() => kept.push(' '),
            c if c.is_control() => {}
            c => kept.push(c),
        }
    }
    kept
}

fn skip_sequence(chars: &mut Chars<'_>) {
    match chars.next() {
        Some('[') => skip_control_sequence(chars),
        Some(']') => skip_operating_command(chars),
        _ => {}
    }
}

fn skip_control_sequence(chars: &mut Chars<'_>) {
    for character in chars.by_ref() {
        if ('\x40'..='\x7e').contains(&character) {
            return;
        }
    }
}

fn skip_operating_command(chars: &mut Chars<'_>) {
    while let Some(character) = chars.next() {
        match character {
            BELL => return,
            ESCAPE => {
                chars.next();
                return;
            }
            _ => {}
        }
    }
}
