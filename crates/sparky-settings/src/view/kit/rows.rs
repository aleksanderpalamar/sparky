use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Alignment, Color, Element, Length};
use sparky_theme::Skin;
use sparky_ui::notice::Notice;

use super::look::border;
use super::reveal::FOCUSED;
use super::tokens::{alpha, size, space};
use crate::app::message::Message;

pub struct Line<'a> {
    label: String,
    note: Option<String>,
    control: Element<'a, Message>,
}

impl<'a> Line<'a> {
    pub fn new(label: impl Into<String>, control: impl Into<Element<'a, Message>>) -> Self {
        Self {
            label: label.into(),
            note: None,
            control: control.into(),
        }
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn view(self, skin: Skin) -> Element<'a, Message> {
        let title = text(self.label).size(size::BODY).color(skin.foreground);
        let (names, padding) = match self.note {
            Some(note) => (
                column![
                    title,
                    text(note).size(size::SMALL).color(skin.dark_foreground)
                ]
                .spacing(space::XXS),
                space::LG,
            ),
            None => (column![title], space::MD),
        };
        let line = row![names.width(Length::Fill), self.control]
            .spacing(space::XXL)
            .align_y(Alignment::Center);
        container(line)
            .width(Length::Fill)
            .padding([padding, space::LG])
            .into()
    }
}

pub fn grouped<'a>(
    skin: Skin,
    items: impl IntoIterator<Item = (&'static str, Element<'a, Message>)>,
) -> Element<'a, Message> {
    let mut current = None;
    let mut body = Column::new().spacing(space::MD).width(Length::Fill);
    for (group, item) in items {
        if current != Some(group) {
            body = body.push(caption(skin, group));
            current = Some(group);
        }
        body = body.push(item);
    }
    body.into()
}

pub fn caption<'a>(skin: Skin, label: &str) -> Element<'a, Message> {
    container(
        text(label.to_uppercase())
            .size(size::CAPTION)
            .color(skin.dark_foreground),
    )
    .padding([space::SM, 0.0])
    .into()
}

pub fn focusable<'a>(
    skin: Skin,
    index: usize,
    focused: bool,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let area = container(mouse_area(cell(skin, focused, content)).on_press(Message::Point(index)));
    match focused {
        true => area.id(FOCUSED).into(),
        false => area.into(),
    }
}

pub fn cell<'a>(skin: Skin, focused: bool, content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fill)
        .style(move |_| {
            let (fill, edge) = match focused {
                true => (
                    skin.fill(alpha::HOVER_FILL),
                    border(skin.fill(alpha::HOVER_BORDER), 1.0),
                ),
                false => (Color::TRANSPARENT, border(Color::TRANSPARENT, 0.0)),
            };
            container::Style {
                background: Some(fill.into()),
                border: edge,
                ..container::Style::default()
            }
        })
        .into()
}

pub fn hint<'a>(skin: Skin, label: impl Into<String>) -> Element<'a, Message> {
    text(label.into())
        .size(size::BODY)
        .color(skin.dark_foreground)
        .into()
}

pub fn notice_bar<'a>(skin: Skin, notice: &Notice) -> Element<'a, Message> {
    let tone = match notice {
        Notice::Info(_) => skin.cyan,
        Notice::Error(_) => skin.urgent,
    };
    container(text(notice.text().to_owned()).size(size::SMALL).color(tone))
        .width(Length::Fill)
        .padding([space::MD, space::LG])
        .style(move |_| container::Style {
            background: Some(Color { a: 0.10, ..tone }.into()),
            ..container::Style::default()
        })
        .into()
}
