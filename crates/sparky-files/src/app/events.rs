use std::time::Duration;

use iced::event::{self, Event, Status};
use iced::mouse;
use iced::{Subscription, time, window};
use sparky_ui::press::{Press, press};

use crate::app::message::Message;
use crate::app::state::Files;

const POLL: Duration = Duration::from_secs(2);

pub fn subscription(_files: &Files) -> Subscription<Message> {
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        event::listen_with(keyboard_message),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
        time::every(POLL).map(|_| Message::Tick),
    ])
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
        return Some(Message::Pointer(position));
    }
    press(event, status).map(|press| match press {
        Press::Escape => Message::Escape,
        Press::Key(key, modifiers) => Message::Key(key, modifiers),
    })
}
