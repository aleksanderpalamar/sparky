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
        caption(skin, "Recebido do roteador"),
        Line::new("Endereço IP", reading(skin, known(lease.address))).view(skin),
        Line::new("Gateway", reading(skin, known(lease.gateway))).view(skin),
        Line::new("DNS", reading(skin, known(lease.dns.join(", ")))).view(skin),
    ]
    .spacing(space::XXS)
    .into()
}

pub fn auto_dns<'a>(skin: Skin, draft: &Draft) -> Element<'a, Message> {
    let options = [
        ("Usar junto com os informados", AutoDns::Use),
        ("Usar somente os informados", AutoDns::Ignore),
    ]
    .into_iter()
    .map(|(label, value)| {
        option_row(
            skin,
            label,
            draft.auto_dns == value,
            Message::NetworkAutoDns(value),
        )
    });
    column![
        caption(skin, "DNS do DHCP"),
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
    let control = text_input(example, value)
        .on_input(move |value| Message::NetworkInput(field, value))
        .on_submit(Message::NetworkSave)
        .size(size::BODY)
        .padding([space::MD, space::XL])
        .width(CONTROL_WIDTH)
        .style(field_look(skin));
    Line::new(label, control).view(skin)
}
