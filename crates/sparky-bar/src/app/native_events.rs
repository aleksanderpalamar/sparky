use super::{
    message::Message,
    panel_state::{AudioClient, BluetoothClient, NetworkClient, PromptClient},
};
use crate::{
    adapters::{
        bluez_control::BluezControl, icons::Icons, network::NetworkManager, prompts::Broker,
        pulse::Pulse,
    },
    domain::audio::Audio,
    ports::network::NetworkControl,
};
use iced::futures::{SinkExt, Stream, StreamExt, stream};
use std::sync::Arc;

pub fn events() -> impl Stream<Item = Message> {
    iced::stream::channel(16, async |mut output| {
        let (prompts, prompt_events) = Broker::new();
        let network = Arc::new(NetworkManager::new(prompts.clone()));
        let bluetooth = Arc::new(BluezControl::new(prompts.clone()));
        let ready = [
            Message::PromptReady(PromptClient(prompts)),
            Message::NetworkReady(NetworkClient(network.clone())),
            Message::BluetoothReady(BluetoothClient(bluetooth)),
        ];
        for message in ready {
            if output.send(message).await.is_err() {
                return;
            }
        }
        let audio_events = match Pulse::start() {
            Ok((client, events)) => {
                if output
                    .send(Message::AudioReady(AudioClient(client)))
                    .await
                    .is_err()
                {
                    return;
                }
                events.map(Message::Audio).boxed()
            }
            Err(error) => {
                stream::once(async move { Message::Audio(Audio::Unavailable(error)) }).boxed()
            }
        };
        let network_events = network.subscribe().map(Message::Network);
        let mut events = stream::select_all(vec![
            audio_events,
            network_events.boxed(),
            prompt_events.map(Message::Prompt).boxed(),
        ]);
        let icons = Icons::current();
        while let Some(message) = events.next().await {
            let indicator = match &message {
                Message::Audio(audio) => Some(super::audio::indicator(audio, &icons)),
                Message::Network(network) => Some(super::network::indicator(network, &icons)),
                _ => None,
            };
            let tray =
                indicator.map(|item| Message::Tray(crate::domain::tray::TrayChange::Added(item)));
            let Some(message) = send_indicator(&mut output, tray, message).await else {
                return;
            };
            if output.send(message).await.is_err() {
                return;
            }
        }
    })
}

async fn send_indicator(
    output: &mut iced::futures::channel::mpsc::Sender<Message>,
    tray: Option<Message>,
    message: Message,
) -> Option<Message> {
    let Some(tray) = tray else {
        return Some(message);
    };
    output.send(tray).await.ok()?;
    Some(message)
}
