use iced::widget::{column, row, rule, slider, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::{action, parts};
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::audio::{Audio, AudioApp, AudioRequest, AudioState, Direction};

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Audio::Ready(audio) = &panels.audio else {
        let Audio::Unavailable(error) = &panels.audio else {
            return text("").into();
        };
        return text(error).size(12).color(skin.foreground).into();
    };
    let output = audio.selected(Direction::Output);
    let heading = match output {
        Some(device) => parts::heading(
            skin,
            "audio-volume-high-symbolic",
            device.label.clone(),
            format!("Saída padrão · {}%", device.volume),
            Some((
                !device.muted,
                Message::AudioRequest(AudioRequest::Mute {
                    direction: Direction::Output,
                    name: device.name.clone(),
                    muted: !device.muted,
                }),
            )),
        ),
        None => parts::heading(
            skin,
            "audio-volume-muted-symbolic",
            "Som".to_owned(),
            "Nenhuma saída disponível".to_owned(),
            None,
        ),
    };
    let mut body = column![heading, parts::section(skin, "SAÍDA")].spacing(12);
    body = body.push(device_section(skin, panels, audio, Direction::Output));
    body = body.push(parts::section(skin, "APLICATIVOS"));
    if audio.applications.is_empty() {
        body = body.push(
            text("Nenhum aplicativo reproduzindo áudio")
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    for app in &audio.applications {
        body = body.push(application(skin, panels, app));
    }
    body.push(rule::horizontal(1))
        .push(parts::section(skin, "MICROFONE"))
        .push(device_section(skin, panels, audio, Direction::Input))
        .into()
}

fn device_section<'a>(
    skin: &Skin,
    panels: &'a Panels,
    audio: &'a AudioState,
    direction: Direction,
) -> Element<'a, Message> {
    let mut body = column![].spacing(6);
    if let Some(toggle) = microphone_toggle(skin, audio, direction) {
        body = body.push(toggle);
    }
    if let Some(device) = audio.selected(direction) {
        let volume = panels
            .volume
            .as_ref()
            .filter(|draft| draft.direction == direction && draft.name == device.name)
            .map_or(device.volume, |draft| draft.percent);
        let label = text(format!("{volume}%"))
            .size(11)
            .color(skin.dark_foreground);
        body = body.push(
            row![
                slider(0..=150, volume, move |value| Message::VolumeDraft(
                    direction, value
                ))
                .on_release(Message::VolumeCommit)
                .style(super::style::volume(*skin)),
                label,
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        );
    }
    if audio.devices(direction).is_empty() {
        body = body.push(
            text("Nenhum dispositivo disponível")
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    for device in audio.devices(direction) {
        let selected = audio.default_name(direction) == Some(device.name.as_str());
        let name = match direction {
            Direction::Output => "audio-speakers-symbolic",
            Direction::Input => "audio-input-microphone-symbolic",
        };
        body = body.push(parts::choice(
            skin,
            name,
            device.label.clone(),
            None,
            selected,
            Some(Message::AudioRequest(AudioRequest::Default {
                direction,
                name: device.name.clone(),
            })),
        ));
    }
    body.into()
}

fn microphone_toggle<'a>(
    skin: &Skin,
    audio: &AudioState,
    direction: Direction,
) -> Option<Element<'a, Message>> {
    if direction != Direction::Input {
        return None;
    }
    let device = audio.selected(Direction::Input)?;
    Some(action(
        skin,
        if device.muted {
            "Ativar microfone"
        } else {
            "Silenciar microfone"
        },
        Message::AudioRequest(AudioRequest::Mute {
            direction,
            name: device.name.clone(),
            muted: !device.muted,
        }),
    ))
}

fn application<'a>(skin: &Skin, panels: &'a Panels, app: &'a AudioApp) -> Element<'a, Message> {
    let volume = panels
        .app_volume
        .as_ref()
        .filter(|draft| draft.index == app.index)
        .map_or(app.volume, |draft| draft.percent);
    let label = row![
        text(&app.label)
            .size(12)
            .color(skin.foreground)
            .width(Length::Fill),
        text(format!("{volume}%"))
            .size(11)
            .color(skin.dark_foreground),
    ];
    if !app.writable {
        return column![
            label,
            text("Controle indisponível")
                .size(11)
                .color(skin.dark_foreground)
        ]
        .spacing(6)
        .into();
    }
    column![
        label,
        slider(0..=150, volume, move |value| Message::AppVolumeDraft(
            app.index, value
        ))
        .on_release(Message::AppVolumeCommit(app.index))
        .style(super::style::volume(*skin)),
    ]
    .spacing(6)
    .into()
}
