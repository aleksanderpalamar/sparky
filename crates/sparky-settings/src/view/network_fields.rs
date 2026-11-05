use iced::Element;
use iced::widget::{Column, column, text_input};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::network::{AutoDns, Draft, Field, Lease};
use crate::view::kit::controls::{option_row, reading};
use crate::view::kit::look::field as field_look;
use crate::view::kit::rows::{Line, caption};
use crate::view::kit::tokens::{CONTROL_WIDTH, size, space};

pub fn lease<'a>(skin: Skin, lease: Lease) -> Element<'a, Message> {
    let known = |value: String| match value.is_empty() {
        true => "—".to_owned(),
        false => value,
    };
    column![
        caption(skin, &crate::i18n::text("network-from-router")),
        Line::new(
            crate::i18n::text("network-ip"),
            reading(skin, known(lease.address))
        )
        .view(skin),
        Line::new(
            crate::i18n::text("network-gateway"),
            reading(skin, known(lease.gateway))
        )
        .view(skin),
        Line::new("DNS", reading(skin, known(lease.dns.join(", ")))).view(skin),
    ]
    .spacing(space::XXS)
    .into()
}

pub fn auto_dns<'a>(skin: Skin, draft: &Draft) -> Element<'a, Message> {
    let options = [
        ("network-auto-dns-use", AutoDns::Use),
        ("network-auto-dns-ignore", AutoDns::Ignore),
    ]
    .into_iter()
    .map(|(label, value)| {
        option_row(
            skin,
            crate::i18n::text(label),
            draft.auto_dns == value,
            Message::NetworkAutoDns(value),
        )
    });
    column![
        caption(skin, &crate::i18n::text("network-dhcp-dns")),
        Column::with_children(options).spacing(space::XXS)
    ]
    .spacing(space::MD)
    .into()
}

pub fn input<'a>(
    skin: Skin,
    label: &'static str,
    example: &'static str,
    field: Field,
    value: &'a str,
) -> Element<'a, Message> {
    let placeholder = crate::i18n::text(example);
    let control = text_input(&placeholder, value)
        .on_input(move |value| Message::NetworkInput(field, value))
        .on_submit(Message::NetworkSave)
        .size(size::BODY)
        .padding([space::MD, space::XL])
        .width(CONTROL_WIDTH)
        .style(field_look(skin));
    Line::new(crate::i18n::text(label), control).view(skin)
}
