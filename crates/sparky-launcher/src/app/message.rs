use std::collections::HashMap;

use iced::Point;
use iced::mouse::ScrollDelta;
use iced_layershell::to_layer_message;

use crate::domain::grid::Move;
use crate::domain::icon::Picture;
use crate::domain::shelf::Shelf;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Icons(HashMap<String, Picture>),
    Typed(String),
    Submit,
    Move(Move),
    NextShelf(isize),
    ShowShelf(Shelf),
    Hover(usize),
    Pick(usize),
    Scrolled(ScrollDelta),
    Pointer(Point),
    AppMenu(String),
    GroupMenu(usize),
    MenuKey,
    Choose(usize),
    AddGroup,
    Named(String),
    ConfirmName,
    Dismiss,
    Close,
}
