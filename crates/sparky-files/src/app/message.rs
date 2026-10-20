use iced::Size;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone)]
pub enum Message {
    Key(Key, Modifiers),
    Escape,
    Typed(String),
    Submit,
    Clicked(usize),
    Activated(usize),
    Scrolled(isize),
    Resized(Size),
    Place(usize),
    Tick,
}
