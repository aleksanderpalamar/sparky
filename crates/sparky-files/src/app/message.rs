use crate::domain::grid::ViewMode;
use iced::Size;
use iced::keyboard::{Key, Modifiers};
use sparky_storage::{Operation, Volume};

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
    View(ViewMode),
    OpenVolume(String),
    UnmountVolume(String),
    VolumesCompleted(Operation, Result<Vec<Volume>, String>),
}
