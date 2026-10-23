use iced::keyboard::{Key, Modifiers};
use sparky_theme::{Palette, TerminalPalette};

use crate::adapters::listener::{Listener, Signal};
use crate::domain::grid::Spot;
use crate::domain::input::Focus;

#[derive(Debug, Clone)]
pub enum Message {
    Connected(Listener),
    Pty(Signal),
    Resized(iced::Size),
    Key(Key, Modifiers, Option<String>, Option<char>),
    Colors(Palette, TerminalPalette),
    Pasted(Option<String>),
    Commit(String),
    Focus(Focus),
    Rescaled(f32),
    Press(Spot),
    Drag(Spot),
    Wheel(f32),
}
