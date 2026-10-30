use iced::event::{self, Event, Status};
use iced::{Subscription, window};
use sparky_ui::press::{Press, press};

use super::keys::{Command, command};
use super::message::Message;
use super::state::Viewer;

pub fn subscription(_viewer: &Viewer) -> Subscription<Message> {
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        event::listen_with(keyboard),
        window::resize_events().map(|(_, size)| Message::Resized(size)),
    ])
}

fn keyboard(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    match press(event, status)? {
        Press::Escape => Some(Message::Command(Command::Quit)),
        Press::Key(key, modifiers) => command(&key, modifiers).map(Message::Command),
    }
}
