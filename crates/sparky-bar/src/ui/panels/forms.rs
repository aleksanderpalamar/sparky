use super::action;
use crate::i18n;
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
use fluent_bundle::FluentArgs;
use iced::{
    Element,
    widget::{checkbox, column, row, text, text_input},
};
use sparky_theme::Skin;

pub fn wifi<'a>(skin: &Skin, wifi: &'a WifiPassword) -> Element<'a, Message> {
    column![
        text(single("connect-to", "network", wifi.network.label()))
            .size(13)
            .color(skin.accent),
        text_input(&i18n::text("wifi-password"), wifi.password.expose())
            .secure(true)
            .on_input(|value| Message::WifiPassword(Secret::new(value)))
            .on_submit(Message::WifiConnect)
            .style(sparky_ui::style::input),
        checkbox(wifi.remember)
            .label(i18n::text("remember-network"))
            .on_toggle(Message::WifiRemember),
        row![
            action(skin, i18n::text("back"), Message::WifiCancel),
            action(skin, i18n::text("connect"), Message::WifiConnect)
        ]
        .spacing(8),
        text(i18n::text("password-managed"))
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
            body = body.push(
                text(single("password-for", "network", ssid.clone()))
                    .size(13)
                    .color(skin.accent),
            );
            body = body.push(input(panels, i18n::text("wifi-password")));
            body = body.push(answers(skin, Answer::Secret(panels.prompt_input.clone())));
        }
        PromptKind::Pin { device, numeric } => {
            let label = if *numeric {
                "numeric-code-hint"
            } else {
                "pin-hint"
            };
            body = body.push(
                text(single("pair-with", "device", device.clone()))
                    .size(13)
                    .color(skin.accent),
            );
            body = body.push(input(panels, i18n::text(label)));
            body = body.push(answers(skin, Answer::Secret(panels.prompt_input.clone())));
        }
        PromptKind::Confirm { device, code } => {
            body = body.push(
                text(single("authorize-device", "device", device.clone()))
                    .size(13)
                    .color(skin.accent),
            );
            if let Some(code) = code {
                body = body.push(
                    text(single("confirm-code", "code", format!("{code:06}")))
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
            body = body.push(text(display_code(code, device)).size(14).color(skin.accent));
            body = body.push(
                text(single("keys-entered", "count", entered.to_string()))
                    .size(12)
                    .color(skin.foreground),
            );
            body = body.push(action(
                skin,
                i18n::text("cancel-pairing"),
                Message::BluetoothRequest(BluetoothRequest::Cancel),
            ));
        }
    }
    Some(body.into())
}

fn input<'a>(panels: &'a Panels, label: String) -> Element<'a, Message> {
    text_input(&label, panels.prompt_input.expose())
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
        action(
            skin,
            i18n::text("cancel"),
            Message::PromptAnswer(Answer::Reject)
        ),
        action(skin, i18n::text("confirm"), Message::PromptAnswer(answer))
    ]
    .spacing(8)
    .into()
}

fn single(key: &str, name: &str, value: String) -> String {
    let mut args = FluentArgs::new();
    args.set(name, value);
    i18n::message(key, Some(&args))
}

fn display_code(code: &str, device: &str) -> String {
    let mut args = FluentArgs::new();
    args.set("code", code.to_owned());
    args.set("device", device.to_owned());
    i18n::message("display-code", Some(&args))
}
