use iced::alignment::Vertical;
use iced::widget::{button as widget_button, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::icons::IconKind;
use crate::view::icon::icon;

pub fn button(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let label = row![
        icon(files, IconKind::Trash, 20.0),
        text("Lixeira").size(13.0)
    ]
    .spacing(6)
    .align_y(Vertical::Center);
    widget_button(label)
        .width(Length::Fill)
        .on_press(Message::OpenTrash)
        .style(move |_, status| style::entry(skin, Highlight::when(files.in_trash()), status))
        .into()
}
