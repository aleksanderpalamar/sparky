use iced::Element;
use iced::widget::{image, space, svg};

use super::style::ICON_SIZE;
use crate::app::message::Message;

pub fn app_icon<'a>(path: Option<&str>) -> Element<'a, Message> {
    match path {
        Some(path) if path.ends_with(".svg") => svg(path).width(ICON_SIZE).height(ICON_SIZE).into(),
        Some(path) => image(path).width(ICON_SIZE).height(ICON_SIZE).into(),
        None => space().width(ICON_SIZE).height(ICON_SIZE).into(),
    }
}
