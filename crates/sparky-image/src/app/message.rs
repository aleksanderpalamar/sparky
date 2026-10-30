use std::path::PathBuf;

use iced::Size;
use sparky_theme::Palette;

use super::keys::Command;
use crate::domain::picture::Picture;
use crate::ports::DecodeError;

#[derive(Debug, Clone)]
pub enum Message {
    Palette(Palette),
    Command(Command),
    Resized(Size),
    Dragged(f32, f32),
    Loaded(PathBuf, Result<Picture, DecodeError>),
}
