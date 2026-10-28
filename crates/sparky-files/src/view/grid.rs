use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, Row, Space, column, container, mouse_area, text};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::view::icon::entry_icon;
use crate::view::style::TILE_HEIGHT;
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};
use sparky_ui::wheel;

pub fn grid(files: &Files) -> Element<'_, Message> {
    let range = files
        .layout
        .range(files.viewport.top(), files.listing.len());
    let rows = range.step_by(files.layout.columns).map(|start| {
        let cells =
            (start..start + files.layout.columns).map(|index| match files.listing.get(index) {
                Some(entry) => tile(files, entry, index),
                None => Space::new().width(Length::FillPortion(1)).into(),
            });
        Row::with_children(cells)
            .spacing(SPACING)
            .width(Length::Fill)
            .into()
    });
    let body: Element<'_, Message> = match files.listing.is_empty() {
        true => text("Pasta vazia")
            .color(files.skin.dark_foreground)
            .size(TEXT_SIZE)
            .into(),
        false => Column::with_children(rows)
            .spacing(SPACING)
            .width(Length::Fill)
            .into(),
    };
    mouse_area(
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(PADDING),
    )
    .on_scroll(|delta| Message::Scrolled(wheel::rows(delta, TILE_HEIGHT + SPACING)))
    .on_right_press(Message::ContextBackground)
    .into()
}

fn tile<'a>(files: &'a Files, entry: &'a Entry, index: usize) -> Element<'a, Message> {
    let skin = files.skin;
    let highlight = Highlight::when(index == files.listing.cursor());
    let path = files.location.join(&entry.name);
    let place = files
        .places
        .iter()
        .find(|place| place.path == path)
        .map(|place| place.label);
    let marked = files.listing.is_marked(entry);
    let name = match marked {
        true => format!("✓ {}", entry.name),
        false => entry.name.clone(),
    };
    let color = match (marked, entry.hidden()) {
        (true, _) => skin.cyan,
        (false, true) => skin.dark_foreground,
        _ => skin.foreground,
    };
    let label = text(name)
        .size(13.0)
        .color(color)
        .width(Length::Fill)
        .height(42.0)
        .align_x(Horizontal::Center)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
    let body = column![entry_icon(files, entry, place, 56.0), label]
        .align_x(Horizontal::Center)
        .spacing(6.0)
        .width(Length::Fill);
    let cell = container(body)
        .padding([8.0, 4.0])
        .width(Length::FillPortion(1))
        .height(TILE_HEIGHT)
        .align_y(Vertical::Center)
        .clip(true)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell)
        .on_press(Message::Clicked(index))
        .on_right_press(Message::ContextEntry(index))
        .on_double_click(Message::Activated(index))
        .into()
}
