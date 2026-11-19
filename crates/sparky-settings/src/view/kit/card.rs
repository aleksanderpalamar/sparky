use iced::widget::{Column, button, column, container, row, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::chrome::separator;
use super::look::border;
use super::tokens::{alpha, size, space};
use crate::app::message::Message;

const OPEN: char = '\u{f0143}';
const CLOSED: char = '\u{f0140}';

pub fn heading<'a>(skin: Skin, title: String, subtitle: String) -> Element<'a, Message> {
    column![
        text(title)
            .size(size::HEADING)
            .color(skin.bright_foreground),
        text(subtitle).size(size::SMALL).color(skin.dark_foreground),
    ]
    .spacing(space::SM)
    .into()
}

pub fn title<'a>(skin: Skin, label: String) -> Element<'a, Message> {
    container(text(label).size(size::BODY).color(skin.bright_foreground))
        .padding([space::SM, 0.0])
        .into()
}

pub fn group<'a>(skin: Skin, rows: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    let mut body = Column::new().width(Length::Fill);
    for (index, item) in rows.into_iter().enumerate() {
        if index > 0 {
            body = body.push(container(separator(skin)).padding([0.0, space::LG]));
        }
        body = body.push(item);
    }
    container(body)
        .width(Length::Fill)
        .padding([space::XS, 0.0])
        .style(move |_| container::Style {
            background: Some(skin.fill(alpha::NORMAL_FILL).into()),
            border: border(skin.fill(alpha::HOVER_BORDER), 1.0),
            ..container::Style::default()
        })
        .into()
}

pub fn fold<'a>(skin: Skin, label: String, open: bool, message: Message) -> Element<'a, Message> {
    let mark = if open { OPEN } else { CLOSED };
    button(
        row![
            text(label)
                .size(size::BODY)
                .color(skin.bright_foreground)
                .width(Length::Fill),
            text(mark.to_string())
                .size(size::ICON)
                .color(skin.dark_foreground),
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([space::SM, 0.0])
    .on_press(message)
    .style(move |_, _| button::Style {
        text_color: skin.foreground,
        ..button::Style::default()
    })
    .into()
}
