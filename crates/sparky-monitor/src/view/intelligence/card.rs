use iced::widget::{column, container, text};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{PADDING, SMALL_TEXT, TITLE_SIZE};

use crate::app::message::Message;
use crate::domain::decision::Tone;

const EDGE: f32 = 0.14;

pub fn card<'a>(skin: Skin, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(PADDING)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(skin.lighter_background)),
            text_color: Some(skin.foreground),
            border: Border {
                color: skin.fill(EDGE),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

pub fn tile<'a>(skin: Skin, label: String, value: String) -> Element<'a, Message> {
    card(
        skin,
        column![
            text(label).size(SMALL_TEXT).color(skin.dark_foreground),
            text(value)
                .size(TITLE_SIZE + 2.0)
                .color(skin.bright_foreground),
        ]
        .spacing(6),
    )
}

pub fn shade(skin: Skin, tone: Tone) -> iced::Color {
    match tone {
        Tone::Positive => skin.accent,
        Tone::Neutral => skin.dark_foreground,
        Tone::Protective => skin.cyan,
        Tone::Negative => skin.urgent,
    }
}
