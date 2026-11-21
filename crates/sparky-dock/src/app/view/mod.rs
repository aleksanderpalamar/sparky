mod dock;
mod menu;
mod picker;
mod picture;
pub mod style;
#[cfg(test)]
mod style_tests;

use iced::widget::Space;
use iced::{Element, window};

use crate::app::message::Message;
use crate::app::state::Dock;

pub fn view(dock: &Dock, window: window::Id) -> Element<'_, Message> {
    if window == dock.window {
        return dock::view(dock);
    }
    match &dock.picker {
        Some(open) if open.window == window => picker::view(dock, &open.picker),
        _ => Space::new().into(),
    }
}
