use std::time::Duration;

use iced::event::{self, Event, Status};
use iced::{Subscription, time, window};
use sparky_ui::press::{Press, press};

use super::keys;
use super::message::Message;
use super::state::Monitor;

const REFRESH: Duration = Duration::from_secs(1);

pub fn subscription(_monitor: &Monitor) -> Subscription<Message> {
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        event::listen_with(keyboard),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
        time::every(REFRESH).map(|_| Message::Tick),
    ])
}

fn keyboard(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Escape),
        Press::Key(key, modifiers) => keys::message(&key, modifiers),
    }
}
