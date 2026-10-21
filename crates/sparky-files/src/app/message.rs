use iced::Size;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
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
