use iced::alignment::Vertical;
use iced::widget::text::IntoFragment;
use iced::widget::{Column, button, column, container, row, scrollable, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use super::style::SECTION_TITLE;
use crate::app::message::Message;
use crate::app::page::Page;

pub fn section<'a>(skin: Skin, title: impl IntoFragment<'a>) -> Element<'a, Message> {
    text(title)
        .size(SECTION_TITLE)
        .color(skin.bright_foreground)
        .into()
}

pub fn link<'a>(skin: Skin, label: &'a str, page: Page) -> Element<'a, Message> {
    button(
        text(format!("{label} ›"))
            .size(SMALL_TEXT)
            .color(skin.accent),
    )
    .padding([4, 0])
    .on_press(Message::Open(page))
    .style(move |_, status| button::Style {
        background: None,
        text_color: skin.accent,
        ..style::entry(skin, style::Highlight::Plain, status)
    })
    .into()
}

pub fn stats<'a>(skin: Skin, items: Vec<(&'a str, String)>) -> Element<'a, Message> {
    let blocks = items.into_iter().map(|(label, value)| {
        column![
            text(label).size(SMALL_TEXT).color(skin.dark_foreground),
            text(value).size(TEXT_SIZE).color(skin.bright_foreground),
        ]
        .spacing(2)
        .into()
    });
    iced::widget::Row::with_children(blocks)
        .spacing(PADDING * 2.0)
        .align_y(Vertical::Top)
        .into()
}

pub fn panel<'a>(skin: Skin, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(PADDING)
        .width(Length::Fill)
        .style(move |_| style::panel(skin))
        .into()
}

pub fn page<'a>(
    skin: Skin,
    title: String,
    subtitle: Option<String>,
    content: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut heading = row![
        text(title)
            .size(SECTION_TITLE + 4.0)
            .color(skin.bright_foreground)
    ]
    .spacing(SPACING)
    .align_y(Vertical::Bottom);
    if let Some(subtitle) = subtitle {
        heading = heading.push(text(subtitle).size(SMALL_TEXT).color(skin.dark_foreground));
    }
    let body = Column::with_children(content).spacing(PADDING);
    scrollable(column![heading, body].spacing(PADDING).padding(PADDING))
        .height(Length::Fill)
        .into()
}

pub fn waiting<'a>(skin: Skin) -> Element<'a, Message> {
    container(
        text("Lendo o sistema…")
            .size(TEXT_SIZE)
            .color(skin.dark_foreground),
    )
    .center(Length::Fill)
    .into()
}
