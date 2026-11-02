use iced::event::{Event, Status};
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::window;
use sparky_ui::press::{Press, press};

use crate::app::{Message, Step};

pub fn message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Close),
        Press::Key(Key::Named(Named::Tab), modifiers) if modifiers.shift() => {
            Some(Message::Move(Step::Back))
        }
        Press::Key(Key::Named(Named::Tab), _) => Some(Message::Move(Step::Forward)),
        Press::Key(Key::Named(Named::Enter), _) => Some(Message::Activate),
        Press::Key(..) => None,
    }
}
