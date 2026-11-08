use iced::widget::{Space, container, row, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::look::surface;
use super::tokens::{alpha, size, space};
use crate::app::message::Message;

const COG: char = '\u{f08bb}';
pub const FOOTER_HEIGHT: f32 = 24.0;

pub fn header<'a>(skin: Skin) -> Element<'a, Message> {
    let line = row![
        text(COG.to_string())
            .size(size::ICON_LARGE)
            .color(skin.accent),
        text(crate::i18n::text("title"))
            .size(size::HEADING)
            .color(skin.bright_foreground),
    ]
    .spacing(space::XL)
    .align_y(Alignment::Center);
    container(line)
        .width(Length::Fill)
        .padding([space::XXL, space::HUGE])
        .into()
}

pub fn separator<'a>(skin: Skin) -> Element<'a, Message> {
    container(Space::new().height(1.0))
        .width(Length::Fill)
        .style(move |_| surface(skin.fill(alpha::HOVER_BORDER)))
        .into()
}

pub fn footer<'a>(skin: Skin, hints: String) -> Element<'a, Message> {
    container(text(hints).size(size::SMALL).color(skin.dark_foreground))
        .width(Length::Fill)
        .height(FOOTER_HEIGHT)
        .padding([0.0, space::HUGE])
        .align_y(Alignment::Center)
        .into()
}
