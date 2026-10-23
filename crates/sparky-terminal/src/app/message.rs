use iced::keyboard::{Key, Modifiers};
use sparky_theme::{Palette, TerminalPalette};

use crate::adapters::listener::{Listener, Signal};
use crate::domain::grid::Spot;

#[derive(Debug, Clone)]
pub enum Message {
    Connected(Listener),
    Pty(Signal),
    Resized(iced::Size),
    Key(Key, Modifiers, Option<String>),
    Colors(Palette, TerminalPalette),
    Pasted(Option<String>),
    Loaded(Signal, Option<String>),
    Press(Spot),
    Drag(Spot),
    Wheel(f32),
}
