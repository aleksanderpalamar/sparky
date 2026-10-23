use std::time::Duration;

use iced::event::{self, Event, Status};
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{Subscription, keyboard, time, window};

use crate::adapters::listener::Listener;
use crate::app::message::Message;
use crate::app::state::Terminal;
use crate::domain::input::Focus;

const COLORS_EVERY: Duration = Duration::from_secs(1);
const BUFFER: usize = 256;

pub fn subscription(_terminal: &Terminal) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run(pty),
        event::listen_with(input),
        time::every(COLORS_EVERY).map(|_| {
            Message::Colors(
                sparky_theme::current_palette(),
                sparky_theme::current_terminal_palette(),
            )
        }),
    ])
}

fn pty() -> impl Stream<Item = Message> {
    iced::stream::channel(BUFFER, async |mut output| {
        let (sender, mut signals) = mpsc::unbounded();
        if output
            .send(Message::Connected(Listener(sender)))
            .await
            .is_err()
        {
            return;
        }
        while let Some(signal) = signals.next().await {
            if output.send(Message::Pty(signal)).await.is_err() {
                return;
            }
        }
    })
}

fn input(event: Event, _status: Status, _window: window::Id) -> Option<Message> {
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            text,
            physical_key,
            ..
        }) => {
            let latin = key.to_latin(physical_key);
            Some(Message::Key(
                key,
                modifiers,
                text.map(|text| text.to_string()),
                latin,
            ))
        }
        Event::Window(window::Event::Resized(size)) => Some(Message::Resized(size)),
        Event::Window(window::Event::Rescaled(scale)) => Some(Message::Rescaled(scale)),
        Event::Window(window::Event::Focused) => Some(Message::Focus(Focus::Gained)),
        Event::Window(window::Event::Unfocused) => Some(Message::Focus(Focus::Lost)),
        _ => None,
    }
}
