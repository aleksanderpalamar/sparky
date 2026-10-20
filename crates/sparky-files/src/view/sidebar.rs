use iced::Element;
use sparky_ui::sidebar::{Entry, sidebar as panel};
use sparky_ui::style::Highlight;

use crate::app::message::Message;
use crate::app::state::Files;

pub fn sidebar(files: &Files) -> Element<'_, Message> {
    let entries = files
        .places
        .iter()
        .enumerate()
        .map(|(index, place)| Entry {
            label: place.label,
            highlight: Highlight::when(place.path == files.location),
            message: Message::Place(index),
        })
        .collect();
    panel(files.skin, "Lugares", entries)
}
