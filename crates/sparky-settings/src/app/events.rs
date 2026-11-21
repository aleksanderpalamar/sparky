use iced::event::{self, Event, Status};
use std::time::Duration;

use iced::{Subscription, window};
use sparky_ui::press::{Press, press};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::section::Section;

pub fn subscription(settings: &Settings) -> Subscription<Message> {
    let performance = match settings.section == Section::Performance {
        true => iced::time::every(Duration::from_secs(2)).map(|_| Message::PerformanceTick),
        false => Subscription::none(),
    };
    Subscription::batch([
        performance,
        sparky_ui::palette::subscription().map(Message::Palette),
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
