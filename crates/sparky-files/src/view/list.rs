use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Color, Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::view::icon::entry_icon;
use crate::view::labels::{self, Tone};
use crate::view::style::{DATE_WIDTH, ROW_HEIGHT, SIZE_WIDTH};
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};
use sparky_ui::wheel;

pub fn list(files: &Files) -> Element<'_, Message> {
    let range = files.viewport.range(files.listing.len());
    let rows = range.filter_map(|index| {
        let entry = files.listing.get(index)?;
        let highlight = Highlight::when(index == files.listing.cursor());
        Some(entry_row(files, entry, index, highlight))
    });
    let body: Element<'_, Message> = match files.listing.is_empty() {
        true => empty(files.skin, files.in_trash(), files.listing.total_len()),
        false => Column::with_children(rows).into(),
    };
    let area = mouse_area(container(body).width(Length::Fill).height(Length::Fill))
        .on_scroll(|delta| Message::Scrolled(wheel::rows(delta, ROW_HEIGHT)))
        .on_right_press(Message::ContextBackground);
    column![titles(files.skin), area]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn entry_row<'a>(
    files: &'a Files,
    entry: &'a Entry,
    row_index: usize,
    highlight: Highlight,
) -> Element<'a, Message> {
    let skin = files.skin;
    let tone = labels::tone(&files.listing, entry);
    let path = files.location.join(&entry.name);
    let place = files
        .places
        .iter()
        .find(|place| place.path == path)
        .map(|place| place.label);
    let line = row![
        entry_icon(files, entry, place, 22.0),
        text(labels::name(entry, tone))
            .size(TEXT_SIZE)
            .color(tone_color(skin, tone))
            .width(Length::Fill),
        text(labels::size(entry))
            .size(SMALL_TEXT)
            .width(SIZE_WIDTH)
            .align_x(Horizontal::Right),
        text(labels::date(entry))
            .size(SMALL_TEXT)
            .width(DATE_WIDTH)
            .align_x(Horizontal::Right),
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
        .on_right_press(Message::ContextEntry(row_index))
        .on_double_click(Message::Activated(row_index))
        .into()
}

fn titles(skin: Skin) -> Element<'static, Message> {
    let title = |label: &'static str| text(label).size(SMALL_TEXT).color(skin.dark_foreground);
    let line = row![
        title("Nome").width(Length::Fill),
        title("Tamanho")
            .width(SIZE_WIDTH)
            .align_x(Horizontal::Right),
        title("Modificado")
            .width(DATE_WIDTH)
            .align_x(Horizontal::Right),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(line)
        .height(ROW_HEIGHT)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .into()
}

fn empty(skin: Skin, trash: bool, total: usize) -> Element<'static, Message> {
    let label = match (trash, total) {
        (true, 0) => "Lixeira vazia",
        (true, _) => "Nenhum item visível",
        (false, _) => "Pasta vazia",
    };
    container(text(label).size(TEXT_SIZE).color(skin.dark_foreground))
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
