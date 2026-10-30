use std::sync::Arc;

use iced::Size;
use iced::widget::image::Handle;
use sparky_theme::Skin;
use sparky_ui::style::STATUS_HEIGHT;

use crate::domain::album::Album;
use crate::domain::viewport::{Area, Viewport};
use crate::ports::{Decoder, Gallery};

pub struct Services {
    pub decoder: Arc<dyn Decoder>,
    pub gallery: Box<dyn Gallery>,
}

#[derive(Debug, Clone)]
pub struct Ready {
    pub handle: Handle,
    pub viewport: Viewport,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum Shown {
    Empty(&'static str),
    Loading,
    Ready(Ready),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decoding {
    Idle,
    Busy,
}

pub struct Viewer {
    pub album: Option<Album>,
    pub shown: Shown,
    pub decoding: Decoding,
    pub size: Size,
    pub skin: Skin,
    pub services: Services,
}

impl Viewer {
    pub fn area(&self) -> Area {
        Area {
            width: self.size.width,
            height: (self.size.height - STATUS_HEIGHT).max(0.0),
        }
    }
}
