use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::look::{border, surface};
use super::tokens::{alpha, size, space};
use crate::app::message::Message;

const COG: char = '\u{f08bb}';
const CLOSE: char = '\u{f0156}';
pub const FOOTER_HEIGHT: f32 = 24.0;

pub fn header<'a>(skin: Skin) -> Element<'a, Message> {
    let line = row![
        text(COG.to_string())
            .size(size::ICON_LARGE)
            .color(skin.accent),
        text(crate::i18n::text("title"))
            .size(size::HEADING)
            .color(skin.bright_foreground),
        Space::new().width(Length::Fill),
        close(skin),
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

fn close<'a>(skin: Skin) -> Element<'a, Message> {
    button(text(CLOSE.to_string()).size(size::ICON))
        .padding([space::XS, space::MD])
        .on_press(Message::Close)
        .style(move |_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let (fill, text_color) = match hovered {
                true => (skin.urgent, skin.background),
                false => (skin.fill(alpha::NORMAL_FILL), skin.foreground),
            };
            button::Style {
                background: Some(fill.into()),
                text_color,
                border: border(skin.fill(alpha::NORMAL_BORDER), 1.0),
                ..button::Style::default()
            }
        })
        .into()
}
