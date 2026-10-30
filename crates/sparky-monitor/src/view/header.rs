use iced::alignment::Vertical;
use iced::widget::{Row, button, container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, HEADER_HEIGHT, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;

pub fn header(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let tabs = Page::ALL.iter().enumerate().map(|(index, page)| {
        let highlight = Highlight::when(*page == monitor.page);
        let label = row![
            text((index + 1).to_string())
                .size(SMALL_TEXT)
                .color(skin.accent),
            text(page.label()).size(TEXT_SIZE),
        ]
        .spacing(6)
        .align_y(Vertical::Center);
        button(label)
            .on_press(Message::Open(*page))
            .style(move |_, status| style::entry(skin, highlight, status))
            .into()
    });
    container(Row::with_children(tabs).spacing(SPACING / 2.0))
        .height(HEADER_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}
