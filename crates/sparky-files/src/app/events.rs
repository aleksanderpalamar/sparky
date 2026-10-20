use std::time::Duration;

use iced::event::{self, Event, Status};
use iced::keyboard::{self, Key, key::Named};
use iced::{Subscription, time, window};

use crate::app::message::Message;
use crate::app::state::Files;

const POLL: Duration = Duration::from_secs(2);

pub fn subscription(_files: &Files) -> Subscription<Message> {
    Subscription::batch([
        event::listen_with(keyboard_message),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
        time::every(POLL).map(|_| Message::Tick),
    ])
}

pub fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };
    match (&key, status) {
        (Key::Named(Named::Escape), _) => Some(Message::Escape),
        (_, Status::Ignored) => Some(Message::Key(key, modifiers)),
        (_, Status::Captured) => None,
    }
}
