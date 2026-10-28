use iced::mouse::{self, Button};

use super::message::Message;
use crate::domain::grid::Spot;
use crate::domain::tabs::TabId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    #[default]
    Idle,
    Selecting(TabId),
}

pub fn update(
    pointer: &mut Pointer,
    id: TabId,
    event: &mouse::Event,
    spot: Option<Spot>,
    over: bool,
    lines: f32,
) -> Option<Message> {
    if matches!(pointer, Pointer::Selecting(previous) if *previous != id) {
        *pointer = Pointer::Idle;
    }
    match (event, *pointer, spot) {
        (mouse::Event::ButtonPressed(Button::Left), _, Some(at)) if over => {
            *pointer = Pointer::Selecting(id);
            Some(Message::Press(id, at))
        }
        (mouse::Event::CursorMoved { .. }, Pointer::Selecting(_), Some(at)) => {
            Some(Message::Drag(id, at))
        }
        (mouse::Event::ButtonReleased(Button::Left), Pointer::Selecting(_), _) => {
            *pointer = Pointer::Idle;
            Some(Message::Selected(id))
        }
        (mouse::Event::ButtonPressed(Button::Middle), _, _) if over => {
            Some(Message::PastePrimary(id))
        }
        (mouse::Event::WheelScrolled { .. }, _, Some(at)) if over => {
            Some(Message::Wheel(id, lines, at))
        }
        _ => None,
    }
}
