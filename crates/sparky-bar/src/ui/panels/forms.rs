use super::action;
use crate::{
    app::{
        message::Message,
        panel_state::{Panels, WifiPassword},
    },
    domain::{
        bluetooth::BluetoothRequest,
        prompt::{Answer, PromptKind},
        secret::Secret,
    },
};
use iced::{
    Element,
    widget::{checkbox, column, row, text, text_input},
};
use sparky_theme::Skin;

pub fn wifi<'a>(skin: &Skin, wifi: &'a WifiPassword) -> Element<'a, Message> {
    column![
        text(format!("Conectar a {}", wifi.network.label()))
            .size(13)
            .color(skin.accent),
        text_input("Senha Wi-Fi", wifi.password.expose())
            .secure(true)
            .on_input(|value| Message::WifiPassword(Secret::new(value)))
            .on_submit(Message::WifiConnect)
            .style(sparky_ui::style::input),
        checkbox(wifi.remember)
            .label("Lembrar esta rede")
            .on_toggle(Message::WifiRemember),
        row![
            action(skin, "Voltar", Message::WifiCancel),
            action(skin, "Conectar", Message::WifiConnect)
        ]
        .spacing(8),
        text("A senha salva é gerenciada pelo NetworkManager.")
            .size(11)
            .color(skin.dark_foreground),
    ]
    .spacing(12)
    .into()
}

pub fn prompt<'a>(skin: &Skin, panels: &'a Panels) -> Option<Element<'a, Message>> {
    let prompt = panels.prompt.as_ref()?;
    let mut body = column![].spacing(12);
    match &prompt.kind {
        PromptKind::Wifi { ssid } => {
            body = body.push(text(format!("Senha de {ssid}")).size(13).color(skin.accent));
            body = body.push(input(panels, "Senha Wi-Fi"));
            body = body.push(answers(skin, Answer::Secret(panels.prompt_input.clone())));
        }
        PromptKind::Pin { device, numeric } => {
            let label = if *numeric {
                "Código de até seis algarismos"
            } else {
                "PIN do dispositivo (1 a 16 caracteres)"
            };
            body = body.push(
                text(format!("Parear com {device}"))
                    .size(13)
                    .color(skin.accent),
            );
            body = body.push(input(panels, label));
            body = body.push(answers(skin, Answer::Secret(panels.prompt_input.clone())));
        }
        PromptKind::Confirm { device, code } => {
            body = body.push(
                text(format!("Autorizar {device}?"))
                    .size(13)
                    .color(skin.accent),
            );
            if let Some(code) = code {
                body = body.push(
                    text(format!("Confirme o código {code:06} no dispositivo"))
                        .size(14)
                        .color(skin.foreground),
                );
            }
            body = body.push(answers(skin, Answer::Accept));
        }
        PromptKind::Display {
            device,
            code,
            entered,
        } => {
            body = body.push(
                text(format!("Digite {code} em {device}"))
                    .size(14)
                    .color(skin.accent),
            );
            body = body.push(
                text(format!(
                    "Confirme no dispositivo · {entered} teclas digitadas"
                ))
                .size(12)
                .color(skin.foreground),
            );
            body = body.push(action(
                skin,
                "Cancelar pareamento",
                Message::BluetoothRequest(BluetoothRequest::Cancel),
            ));
        }
    }
    Some(body.into())
}

fn input<'a>(panels: &'a Panels, label: &'static str) -> Element<'a, Message> {
    text_input(label, panels.prompt_input.expose())
        .secure(true)
        .on_input(|value| Message::PromptInput(Secret::new(value)))
        .on_submit(Message::PromptAnswer(Answer::Secret(
            panels.prompt_input.clone(),
        )))
        .style(sparky_ui::style::input)
        .into()
}

fn answers<'a>(skin: &Skin, answer: Answer) -> Element<'a, Message> {
    row![
        action(skin, "Cancelar", Message::PromptAnswer(Answer::Reject)),
        action(skin, "Confirmar", Message::PromptAnswer(answer))
    ]
    .spacing(8)
    .into()
}
