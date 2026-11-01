use std::ops::RangeInclusive;

use iced::alignment::Horizontal;
use iced::widget::{combo_box, row, slider, text, toggler};
use iced::{Alignment, Element};
use sparky_theme::Skin;

use super::look::{field, menu, rail, switch as switch_look};
use super::tokens::{CONTROL_WIDTH, size, space};
use crate::app::message::Message;
use crate::app::state::Target;

const SWITCH_SIZE: f32 = 22.0;
const READING_WIDTH: f32 = 62.0;

pub struct Level {
    pub target: Target,
    pub range: RangeInclusive<i32>,
    pub step: i32,
    pub value: i32,
}

pub fn switch<'a>(skin: Skin, on: bool, message: Message) -> Element<'a, Message> {
    toggler(on)
        .on_toggle(move |_| message.clone())
        .size(SWITCH_SIZE)
        .style(switch_look(skin))
        .into()
}

pub fn locked_switch<'a>(skin: Skin, on: bool) -> Element<'a, Message> {
    toggler(on)
        .size(SWITCH_SIZE)
        .style(switch_look(skin))
        .into()
}

pub fn level<'a>(skin: Skin, level: Level, reading: String) -> Element<'a, Message> {
    let Level {
        target,
        range,
        step,
        value,
    } = level;
    row![
        slider(range, value, move |chosen| Message::Slide(target, chosen))
            .step(step)
            .on_release(Message::Release)
            .width(CONTROL_WIDTH - READING_WIDTH - space::XL)
            .style(rail(skin)),
        text(reading)
            .size(size::SMALL)
            .width(READING_WIDTH)
            .align_x(Horizontal::Right)
            .color(skin.dark_foreground),
    ]
    .spacing(space::XL)
    .align_y(Alignment::Center)
    .into()
}

pub fn picker<'a>(
    skin: Skin,
    state: &'a combo_box::State<String>,
    current: &str,
    on_selected: impl Fn(String) -> Message + 'static,
) -> Element<'a, Message> {
    combo_box(state, "selecione", Some(&current.to_owned()), on_selected)
        .size(size::BODY)
        .padding([space::MD, space::XL])
        .width(CONTROL_WIDTH)
        .input_style(field(skin))
        .menu_style(menu(skin))
        .into()
}
