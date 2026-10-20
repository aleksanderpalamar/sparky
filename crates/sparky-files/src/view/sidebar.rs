use iced::alignment::Vertical;
use iced::widget::{Column, button, column, container, row, text};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::view::style::{self, Highlight, PADDING, SIDEBAR_WIDTH, SMALL_TEXT, SPACING, TEXT_SIZE};

pub fn sidebar(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let places = files.places.iter().enumerate().map(|(index, place)| {
        let highlight = match place.path == files.location {
            true => Highlight::Selected,
            false => Highlight::Plain,
        };
        let line = row![
            text((index + 1).to_string()).size(SMALL_TEXT).color(skin.accent).width(18),
            text(place.label).size(TEXT_SIZE),
        ]
        .spacing(SPACING)
        .align_y(Vertical::Center);
        button(line)
            .width(Length::Fill)
            .on_press(Message::Place(index))
            .style(move |_, status| style::place(skin, highlight, status))
            .into()
    });
    let content = column![
        text("Lugares").size(SMALL_TEXT).color(skin.dark_foreground),
        Column::with_children(places).spacing(2),
    ]
    .spacing(SPACING);
    container(content)
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .padding(PADDING)
        .style(move |_| style::panel(skin))
        .into()
}
