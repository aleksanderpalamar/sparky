use iced::event::{Event, Status};
use iced::window;
use sparky_ui::press::{Press, press};

use crate::app::Message;

pub fn message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Escape),
        Press::Key(..) => None,
    }
}
