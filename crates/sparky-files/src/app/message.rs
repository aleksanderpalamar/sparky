use crate::domain::context::Action;
use crate::domain::grid::ViewMode;
use crate::domain::thumbnail::Thumbnail;
use crate::ports::EmptyReport;
use iced::keyboard::{Key, Modifiers};
use iced::{Point, Size};
use sparky_storage::{Operation, Volume};
use std::path::PathBuf;
use std::time::SystemTime;

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
    Pointer(Point),
    ContextEntry(usize),
    ContextBackground,
    ContextAction(Action),
    ContextDismiss,
    OpenWithChoice(String),
    ArchiveCompleted(PathBuf, Result<(), String>),
    PreviewReady(PathBuf, Option<SystemTime>, Option<Thumbnail>),
    Resized(Size),
    Place(usize),
    Trash(TrashEvent),
    TrashEmptied(Result<EmptyReport, String>),
    Tick,
    View(ViewMode),
    OpenVolume(String),
    UnmountVolume(String),
    VolumesCompleted(Operation, Result<Vec<Volume>, String>),
}

#[derive(Debug, Clone, Copy)]
pub enum TrashEvent {
    Open,
    RestoreSelected,
    RestoreAll,
    Empty,
}
