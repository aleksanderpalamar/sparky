use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Color, Element, Length, mouse};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::view::labels::{self, Tone};
use crate::view::style::{self, DATE_WIDTH, Highlight, PADDING, ROW_HEIGHT, SIZE_WIDTH, SMALL_TEXT, SPACING, TEXT_SIZE};

const WHEEL_LINES: f32 = 3.0;

pub fn list(files: &Files) -> Element<'_, Message> {
    let range = files.viewport.range(files.listing.len());
    let top = range.start;
    let rows = range.filter_map(|index| {
        let entry = files.listing.get(index)?;
        let highlight = match index == files.listing.cursor() {
            true => Highlight::Selected,
            false => Highlight::Plain,
        };
        Some(entry_row(files, entry, index - top, highlight))
    });
    let body: Element<'_, Message> = match files.listing.is_empty() {
        true => empty(files.skin),
        false => Column::with_children(rows).into(),
    };
    let area = mouse_area(container(body).width(Length::Fill).height(Length::Fill)).on_scroll(scrolled);
    column![titles(files.skin), area].width(Length::Fill).height(Length::Fill).into()
}

fn entry_row<'a>(files: &'a Files, entry: &'a Entry, row_index: usize, highlight: Highlight) -> Element<'a, Message> {
    let skin = files.skin;
    let tone = labels::tone(&files.listing, entry);
    let line = row![
        text(labels::name(entry, tone)).size(TEXT_SIZE).color(tone_color(skin, tone)).width(Length::Fill),
        text(labels::size(entry)).size(SMALL_TEXT).width(SIZE_WIDTH).align_x(Horizontal::Right),
        text(labels::date(entry)).size(SMALL_TEXT).width(DATE_WIDTH).align_x(Horizontal::Right),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    let cell = container(line)
        .height(ROW_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell)
        .on_press(Message::Clicked(row_index))
        .on_double_click(Message::Activated(row_index))
        .into()
}

fn titles(skin: Skin) -> Element<'static, Message> {
    let title = |label: &'static str| text(label).size(SMALL_TEXT).color(skin.dark_foreground);
    let line = row![
        title("Nome").width(Length::Fill),
        title("Tamanho").width(SIZE_WIDTH).align_x(Horizontal::Right),
        title("Modificado").width(DATE_WIDTH).align_x(Horizontal::Right),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(line)
        .height(ROW_HEIGHT)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .into()
}

fn empty(skin: Skin) -> Element<'static, Message> {
    container(text("Pasta vazia").size(TEXT_SIZE).color(skin.dark_foreground))
        .padding(PADDING)
        .into()
}

fn tone_color(skin: Skin, tone: Tone) -> Color {
    match tone {
        Tone::Directory => skin.accent,
        Tone::File => skin.foreground,
        Tone::Hidden => skin.dark_foreground,
        Tone::Marked => skin.cyan,
    }
}

fn scrolled(delta: mouse::ScrollDelta) -> Message {
    let lines = match delta {
        mouse::ScrollDelta::Lines { y, .. } => y * WHEEL_LINES,
        mouse::ScrollDelta::Pixels { y, .. } => y / ROW_HEIGHT,
    };
    Message::Scrolled(-lines.round() as isize)
}
