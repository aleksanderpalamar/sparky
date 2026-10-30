use iced::widget::{Text, button, column, row, text, text_input};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::network::{AutoDns, Draft, Field, Lease};

const DETAIL_WIDTH: f32 = 140.0;

pub fn caption<'a>(settings: &Settings, label: &'static str) -> Text<'a> {
    text(label)
        .size(SMALL_TEXT)
        .color(settings.context.skin.dark_foreground)
}

pub fn lease<'a>(settings: &Settings, lease: Lease) -> Element<'a, Message> {
    let dns = lease.dns.join(", ");
    column![
        caption(settings, "Recebido do roteador"),
        detail(settings, "Endereço IP", lease.address),
        detail(settings, "Gateway", lease.gateway),
        detail(settings, "DNS", dns),
    ]
    .spacing(4)
    .into()
}

fn detail<'a>(settings: &Settings, label: &'static str, value: String) -> Element<'a, Message> {
    let value = if value.is_empty() {
        "—".to_owned()
    } else {
        value
    };
    row![
        text(label)
            .size(TEXT_SIZE)
            .color(settings.context.skin.dark_foreground)
            .width(Length::Fixed(DETAIL_WIDTH)),
        text(value).size(TEXT_SIZE),
    ]
    .spacing(SPACING)
    .into()
}

pub fn auto_dns<'a>(settings: &Settings, draft: &Draft) -> Element<'a, Message> {
    let options = [
        ("Usar junto com os informados", AutoDns::Use),
        ("Usar somente os informados", AutoDns::Ignore),
    ]
    .into_iter()
    .fold(row![].spacing(SPACING), |options, (label, value)| {
        options.push(choice(
            settings,
            label,
            draft.auto_dns == value,
            Message::NetworkAutoDns(value),
        ))
    });
    column![caption(settings, "DNS do DHCP"), options]
        .spacing(4)
        .into()
}

pub fn choice<'a>(
    settings: &Settings,
    label: &'static str,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    button(text(label).size(TEXT_SIZE))
        .on_press(message)
        .style(move |_, status| style::entry(skin, Highlight::when(selected), status))
        .into()
}

pub fn input<'a>(
    settings: &Settings,
    label: &'static str,
    placeholder: &'static str,
    field: Field,
    value: &'a str,
) -> Element<'a, Message> {
    column![
        caption(settings, label),
        text_input(placeholder, value)
            .on_input(move |value| Message::NetworkInput(field, value))
            .on_submit(Message::NetworkSave)
            .size(TEXT_SIZE)
            .style(style::input),
    ]
    .spacing(4)
    .into()
}
