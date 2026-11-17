use std::path::PathBuf;

use iced::Point;

use crate::domain::application::{AppChoices, Remember};
use crate::domain::context::Target;

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub target: Target,
    pub name: Option<String>,
    pub point: Point,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWith {
    pub path: PathBuf,
    pub choices: AppChoices,
    pub query: String,
    pub selected: Option<String>,
    pub remember: Remember,
}
