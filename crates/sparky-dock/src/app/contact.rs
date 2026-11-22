use iced::{Event, Point, event, mouse, window};

use super::message::Message;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Contact {
    Entered,
    Left,
    Moved(Point),
    Released,
}

pub fn contact(event: &Event) -> Option<Contact> {
    match event {
        Event::Mouse(mouse::Event::CursorEntered) => Some(Contact::Entered),
        Event::Mouse(mouse::Event::CursorLeft) => Some(Contact::Left),
        Event::Mouse(mouse::Event::CursorMoved { position }) => Some(Contact::Moved(*position)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => Some(Contact::Released),
        _ => None,
    }
}

pub fn surface(event: Event, _: event::Status, window: window::Id) -> Option<Message> {
    contact(&event).map(|contact| Message::Surface(window, contact))
}
