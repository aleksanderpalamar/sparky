use iced::widget::{Row, button, container, row, text, text_input};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;

use super::look::{field, frame, selectable};
use super::tokens::{CONTROL_WIDTH, GLYPH_WIDTH, alpha, size, space};
use crate::app::message::Message;

const CHECK: char = '\u{f012c}';

pub fn reading<'a>(skin: Skin, value: impl Into<String>) -> Element<'a, Message> {
    text(value.into())
        .size(size::BODY)
        .color(skin.dark_foreground)
        .into()
}

pub fn boxed<'a>(skin: Skin, value: impl Into<String>, message: Message) -> Element<'a, Message> {
    button(text(value.into()).size(size::BODY))
        .width(CONTROL_WIDTH)
        .padding([space::MD, space::XL])
        .on_press(message)
        .style(frame(skin, alpha::HOVER_BORDER))
        .into()
}

pub fn editor<'a>(skin: Skin, value: &str, width: impl Into<Length>) -> Element<'a, Message> {
    text_input("", value)
        .id(PROMPT)
        .on_input(Message::Typed)
        .on_submit(Message::Submit)
        .size(size::BODY)
        .padding([space::MD, space::XL])
        .width(width)
        .style(field(skin))
        .into()
}

pub fn action<'a>(skin: Skin, label: impl Into<String>, message: Message) -> Element<'a, Message> {
    button(text(label.into()).size(size::BODY))
        .padding([space::MD, space::XL])
        .on_press(message)
        .style(frame(skin, alpha::NORMAL_BORDER))
        .into()
}

pub fn option_row<'a>(
    skin: Skin,
    label: impl Into<String>,
    selected: bool,
    message: Message,
) -> Element<'a, Message> {
    let mark = match selected {
        true => CHECK,
        false => ' ',
    };
    let line = row![
        text(mark.to_string())
            .size(size::ICON)
            .width(GLYPH_WIDTH)
            .color(skin.accent),
        text(label.into()).size(size::BODY).color(skin.foreground),
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center);
    button(centered(line))
        .width(Length::Fill)
        .height(space::ROW_HEIGHT)
        .padding([0.0, space::LG])
        .on_press(message)
        .style(selectable(skin, selected))
        .into()
}

pub fn centered<'a>(line: Row<'a, Message>) -> Element<'a, Message> {
    container(line)
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .into()
}
