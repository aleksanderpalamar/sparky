use iced::Size;
use iced::keyboard::{Key, Modifiers};

use crate::domain::section::Section;
use crate::domain::value::Step;

#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Key(Key, Modifiers),
    Escape,
    Typed(String),
    Submit,
    Section(Section),
    NetworkSelect(usize),
    NetworkMethod(crate::domain::network::Method),
    NetworkInput(crate::domain::network::Field, String),
    NetworkSave,
    Point(usize),
    Nudge(usize, Step),
    Activate(usize),
    Remove(usize),
    Scrolled(isize),
    Resized(Size),
}
