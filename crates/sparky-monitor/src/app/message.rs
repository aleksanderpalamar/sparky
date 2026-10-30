use iced::Size;
use sparky_theme::Palette;

use super::page::Page;
use super::state::Focus;
use crate::domain::apps::AppEntry;
use crate::domain::snapshot::Snapshot;
use crate::domain::table::Column;
use crate::ports::Stop;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Process(u32),
    App(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Palette(Palette),
    Resized(Size),
    Tick,
    Sampled(Box<Snapshot>),
    Catalog(Vec<AppEntry>),
    Open(Page),
    SortBy(Column),
    Select(Selection),
    Move(Move),
    Scrolled(Focus, isize),
    SwitchFocus,
    Ask(Stop),
    Confirm,
    Escape,
    Quit,
}
