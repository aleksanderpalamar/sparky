use std::thread;
use std::time::Duration;

use iced::futures::channel::mpsc::{UnboundedSender, unbounded};
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::{Subscription, time};
use sparky_ipc::Client;

use crate::app::message::Message;
use crate::app::state::Bar;

const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const CLOCK_INTERVAL: Duration = Duration::from_secs(1);

pub fn subscription(_bar: &Bar) -> Subscription<Message> {
    Subscription::batch([
        Subscription::run(sparky_events),
        time::every(CLOCK_INTERVAL).map(|_| Message::Tick),
    ])
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
