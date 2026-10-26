use iced::widget::{column, row, rule, slider, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::action;
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::audio::{Audio, AudioRequest, AudioState, Direction};

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Audio::Ready(audio) = &panels.audio else {
        let Audio::Unavailable(error) = &panels.audio else {
            return text("").into();
        };
        return text(error).size(12).color(skin.foreground).into();
    };
    column![
        section(skin, panels, audio, Direction::Output),
        rule::horizontal(1),
        section(skin, panels, audio, Direction::Input)
    ]
    .spacing(14)
    .into()
}

fn section<'a>(
    skin: &Skin,
    panels: &'a Panels,
    audio: &'a AudioState,
    direction: Direction,
) -> Element<'a, Message> {
    let title = match direction {
        Direction::Output => "Saída de som",
        Direction::Input => "Microfone",
    };
    let mut body = column![text(title).size(13).color(skin.accent)].spacing(8);
    if let Some(device) = audio.selected(direction) {
        let volume = panels
            .volume
            .as_ref()
            .filter(|draft| draft.direction == direction && draft.name == device.name)
            .map_or(device.volume, |draft| draft.percent);
        let mute = AudioRequest::Mute {
            direction,
            name: device.name.clone(),
            muted: !device.muted,
        };
        let label = match device.muted {
            true => "Ativar som",
            false => "Silenciar",
        };
        body = body.push(
            row![
                text(format!("{volume}%"))
                    .size(12)
                    .color(skin.foreground)
                    .width(Length::Fill),
                action(skin, label, Message::AudioRequest(mute))
            ]
            .spacing(8),
        );
        body = body.push(
            slider(0..=150, volume, move |value| {
                Message::VolumeDraft(direction, value)
            })
            .on_release(Message::VolumeCommit)
            .style(super::style::volume(*skin)),
        );
    }
    if audio.devices(direction).is_empty() {
        body = body.push(
            text("Nenhum dispositivo disponível")
                .size(12)
                .color(skin.dark_foreground),
        );
    }
    for device in audio.devices(direction) {
        let selected = audio.default_name(direction) == Some(device.name.as_str());
        let label = format!("{} {}", if selected { "●" } else { "○" }, device.label);
        body = body.push(action(
            skin,
            label,
            Message::AudioRequest(AudioRequest::Default {
                direction,
                name: device.name.clone(),
            }),
        ));
    }
    body.into()
}
