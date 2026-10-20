use iced::Size;
use iced::keyboard::{Key, Modifiers};

use crate::domain::section::Section;
use crate::domain::value::Step;

#[derive(Debug, Clone)]
pub enum Message {
    Key(Key, Modifiers),
    Escape,
    Typed(String),
    Submit,
    Section(Section),
    Point(usize),
    Nudge(usize, Step),
    Activate(usize),
    Remove(usize),
    Scrolled(isize),
    Resized(Size),
}
