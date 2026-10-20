use iced::alignment::Vertical;
use iced::widget::{Column, button, column, container, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::style::{self, Highlight, PADDING, SIDEBAR_WIDTH, SMALL_TEXT, SPACING, TEXT_SIZE};

const NUMBER_WIDTH: f32 = 18.0;

pub struct Entry<'a, M> {
    pub label: &'a str,
    pub highlight: Highlight,
    pub message: M,
}

pub fn sidebar<'a, M: Clone + 'a>(skin: Skin, title: &'a str, entries: Vec<Entry<'a, M>>) -> Element<'a, M> {
    let items = entries.into_iter().enumerate().map(|(index, entry)| item(skin, index, entry));
    let content = column![
        text(title).size(SMALL_TEXT).color(skin.dark_foreground),
        Column::with_children(items).spacing(2),
    ]
    .spacing(SPACING);
    container(content)
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .padding(PADDING)
        .style(move |_| style::panel(skin))
        .into()
}

fn item<'a, M: Clone + 'a>(skin: Skin, index: usize, entry: Entry<'a, M>) -> Element<'a, M> {
    let line = row![
        text((index + 1).to_string()).size(SMALL_TEXT).color(skin.accent).width(NUMBER_WIDTH),
        text(entry.label).size(TEXT_SIZE),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    let highlight = entry.highlight;
    button(line)
        .width(Length::Fill)
        .on_press(entry.message)
        .style(move |_, status| style::entry(skin, highlight, status))
        .into()
}
