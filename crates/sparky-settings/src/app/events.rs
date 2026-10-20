use iced::event::{self, Event, Status};
use iced::{Subscription, window};
use sparky_ui::press::{Press, press};

use crate::app::message::Message;
use crate::app::state::Settings;

pub fn subscription(_settings: &Settings) -> Subscription<Message> {
    Subscription::batch([
        event::listen_with(keyboard_message),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
    ])
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    press(event, status).map(|press| match press {
        Press::Escape => Message::Escape,
        Press::Key(key, modifiers) => Message::Key(key, modifiers),
    })
}
