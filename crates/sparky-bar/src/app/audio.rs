use iced::Task;

use crate::adapters::icons::Icons;
use crate::app::{message::Message, state::Bar};
use crate::domain::{
    audio::{Audio, AudioRequest},
    indicator::{Role, Status},
    panel::PanelKind,
    tray::{PrimaryAction, TrayIcon, TrayItem},
};

pub const ADDRESS: &str = "sparky.audio";

pub fn indicator(audio: &Audio, icons: &Icons) -> TrayItem {
    let (name, title) = match &audio {
        Audio::Ready(state) => (state.icon(), "Áudio".to_owned()),
        Audio::Unavailable(error) => ("audio-volume-muted-symbolic", error.clone()),
    };
    TrayItem {
        address: ADDRESS.to_owned(),
        title,
        icon: icons.resolve(name, None).unwrap_or(TrayIcon::Missing),
        role: Role::Audio,
        status: Status::Active,
        primary_action: PrimaryAction::Activate,
        menu_path: None,
        menu: Vec::new(),
    }
}

pub fn request(bar: &mut Bar, request: AudioRequest) -> Task<Message> {
    let Some(client) = bar.panels.audio_client.clone() else {
        return Task::none();
    };
    let id = bar.panels.begin(PanelKind::Audio);
    Task::perform(async move { client.0.send(request).await }, move |result| {
        Message::OperationFinished(id, PanelKind::Audio, result)
    })
}
