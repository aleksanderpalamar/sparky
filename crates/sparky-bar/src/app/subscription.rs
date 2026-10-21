use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc::{UnboundedSender, unbounded};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{Subscription, keyboard, time};
use sparky_ipc::Client;

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::app::tray_subscription::tray_events;

const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const CLOCK_INTERVAL: Duration = Duration::from_secs(1);

pub fn subscription(_bar: &Bar) -> Subscription<Message> {
    Subscription::batch([
        sparky_ui::palette::subscription().map(Message::Palette),
        Subscription::run(sparky_events),
        Subscription::run(tray_events),
        time::every(CLOCK_INTERVAL).map(|_| Message::Tick),
        keyboard::listen().filter_map(close_on_escape),
    ])
}

fn close_on_escape(event: keyboard::Event) -> Option<Message> {
    match event {
        keyboard::Event::KeyPressed { key: keyboard::Key::Named(keyboard::key::Named::Escape), .. } => {
            Some(Message::CloseMenu)
        }
        _ => None,
    }
}

fn sparky_events() -> impl Stream<Item = Message> {
    iced::stream::channel(16, async |mut output| {
        let (sender, mut changes) = unbounded();
        thread::spawn(move || forward_changes(&sender));
        while changes.next().await.is_some() {
            if output.send(Message::WorkspacesChanged).await.is_err() {
                return;
            }
        }
    })
}

fn forward_changes(sender: &UnboundedSender<()>) {
    while !sender.is_closed() {
        if let Ok(events) = Client::from_env().and_then(Client::subscribe) {
            for _ in std::iter::once(()).chain(events.map(drop)) {
                if sender.unbounded_send(()).is_err() {
                    return;
                }
            }
        }
        thread::sleep(RECONNECT_DELAY);
    }
}

#[cfg(test)]
mod tests {
    use iced::keyboard::{self, Key, Location, Modifiers, key};

    use super::close_on_escape;
    use crate::app::tray_test_support::{bar, messages, opened, removes};
    use crate::app::update::update;

    fn pressed(named: key::Named, code: key::Code) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: Key::Named(named),
            modified_key: Key::Named(named),
            physical_key: key::Physical::Code(code),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn escape_percorre_o_tratador_e_fecha_a_superficie_do_menu() {
        let mut bar = bar();
        let window = opened(&mut bar, ":1.1");
        let event = pressed(key::Named::Escape, key::Code::Escape);
        let output = close_on_escape(event).map(|message| messages(update(&mut bar, message))).unwrap_or_default();
        assert!(removes(&output, window));
        assert!(bar.tray().menu().is_none());
    }

    #[test]
    fn outras_teclas_e_modificadores_nao_fecham_o_menu() {
        assert!(close_on_escape(pressed(key::Named::Enter, key::Code::Enter)).is_none());
        assert!(close_on_escape(keyboard::Event::ModifiersChanged(Modifiers::SHIFT)).is_none());
    }
}
