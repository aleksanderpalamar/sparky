use crate::app::jobs::JobEvent;
use crate::domain::context::Action;
use crate::domain::drop::DropTarget;
use crate::domain::grid::ViewMode;
use crate::domain::order::SortKey;
use crate::domain::search::SearchEvent;
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation};
use crate::domain::thumbnail::Thumbnail;
use crate::ports::EmptyReport;
use iced::keyboard::{Key, Modifiers};
use iced::{Point, Size, window};
use iced_layershell::to_layer_message;
use sparky_storage::{Operation, Volume};
use std::path::PathBuf;
use std::time::SystemTime;

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    Moved(window::Id, Point),
    Sized(window::Id, Size),
    Opened(window::Id, Size),
    Closed(window::Id),
    Palette(sparky_theme::Palette),
    Key(Key, Modifiers),
    Escape,
    Typed(String),
    Submit,
    Clicked(usize),
    Activated(usize),
    Scrolled(isize),
    Pointer(Point),
    Released,
    BookmarkPressed(usize),
    Hover(DropTarget),
    Unhover(DropTarget),
    Modifiers(Modifiers),
    DragRefused(String),
    ToastUndo,
    ToastDismiss,
    ToastOpenFolder,
    DeleteConfirm,
    DeleteCancel,
    ContextEntry(usize),
    ContextBackground,
    ContextBookmark(usize),
    ContextAction(Action),
    ContextDismiss,
    OpenWithChoice(String),
    OpenWithSelect(String),
    OpenWithQuery(String),
    OpenWithRemember(bool),
    OpenWithConfirm,
    ArchiveCompleted(PathBuf, Result<(), String>),
    PreviewReady(PathBuf, Option<SystemTime>, Option<Thumbnail>),
    Resized(Size),
    Place(usize),
    Sidebar(SidebarLocation),
    SidebarItem(usize),
    NetworkLoaded(String, Result<Vec<SidebarItem>, String>),
    FavoriteSelected,
    RemoveFavorite(usize),
    Trash(TrashEvent),
    TrashEmptied(Result<EmptyReport, String>),
    Tick,
    FolderChanged,
    Job(JobEvent),
    View(ViewMode),
    Sort(SortKey),
    Search(u64, SearchEvent),
    Tab(usize),
    CloseTab(usize),
    OpenVolume(String),
    UnmountVolume(String),
    EjectVolume(String),
    VolumesCompleted(Operation, Result<Vec<Volume>, String>),
    Chooser(crate::app::chooser::ChooserEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum TrashEvent {
    Open,
    OpenOther(usize),
    RestoreSelected,
    RestoreAll,
    Empty,
}
