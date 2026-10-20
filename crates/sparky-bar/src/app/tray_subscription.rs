use std::sync::Arc;

use iced::futures::{SinkExt, Stream};
use iced::futures::channel::mpsc::Sender;
use system_tray::client::Client;
use tokio::sync::broadcast::error::RecvError;

use crate::adapters::tray_events::{changes, item_of};
use crate::app::message::Message;
use crate::app::tray_state::TrayClient;
use crate::domain::tray::TrayChange;

pub fn tray_events() -> impl Stream<Item = Message> {
    iced::stream::channel(64, async |mut output| {
        let client = match Client::new().await {
            Ok(client) => Arc::new(client),
            Err(error) => {
                eprintln!("sparky-bar: bandeja indisponível: {error}");
                return;
            }
        };
        let mut events = client.subscribe();
        if output.send(Message::TrayReady(TrayClient(client.clone()))).await.is_err() {
            return;
        }
        for change in current_items(&client) {
            if output.send(Message::Tray(change)).await.is_err() {
                return;
            }
        }
        loop {
            match events.recv().await {
                Ok(event) => {
                    if !forward(&mut output, changes(event)).await {
                        return;
                    }
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            }
        }
    })
}

async fn forward(output: &mut Sender<Message>, changes: Vec<TrayChange>) -> bool {
    for change in changes {
        if output.send(Message::Tray(change)).await.is_err() {
            return false;
        }
    }
    true
}

fn current_items(client: &Client) -> Vec<TrayChange> {
    let items = client.items();
    let Ok(items) = items.lock() else {
        return Vec::new();
    };
    items
        .iter()
        .map(|(address, (item, menu))| TrayChange::Added(item_of(address.clone(), item.clone(), menu.as_ref())))
        .collect()
}
