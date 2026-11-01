use iced::widget::{Column, button, container, row, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::controls::centered;
use super::look::{selectable, surface};
use super::tokens::{GLYPH_WIDTH, SIDEBAR_WIDTH, size, space};
use crate::app::message::Message;
use crate::domain::section::Section;

pub fn sidebar<'a>(skin: Skin, current: Section) -> Element<'a, Message> {
    let items = Section::ALL
        .iter()
        .map(|section| item(skin, *section, *section == current));
    container(
        Column::with_children(items)
            .spacing(space::XXS)
            .padding(space::LG),
    )
    .width(SIDEBAR_WIDTH)
    .height(Length::Fill)
    .style(move |_| surface(skin.lighter_background))
    .into()
}

fn item<'a>(skin: Skin, section: Section, selected: bool) -> Element<'a, Message> {
    let line = row![
        text(glyph(section).to_string())
            .size(size::ICON)
            .width(GLYPH_WIDTH)
            .color(skin.foreground),
        text(section.label())
            .size(size::BODY)
            .color(skin.foreground),
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center);
    button(centered(line))
        .width(Length::Fill)
        .height(space::ROW_HEIGHT)
        .padding([0.0, space::LG])
        .on_press(Message::Section(section))
        .style(selectable(skin, selected))
        .into()
}

pub const fn glyph(section: Section) -> char {
    match section {
        Section::Appearance => '\u{f0614}',
        Section::Keyboard => '\u{f030c}',
        Section::Apps => '\u{f003b}',
        Section::Autostart => '\u{f14de}',
        Section::Shortcuts => '\u{f09f9}',
        Section::Monitors => '\u{f0379}',
        Section::Power => '\u{f0425}',
        Section::DateTime => '\u{f0150}',
        Section::Region => '\u{f05ca}',
        Section::Users => '\u{f000f}',
        Section::Media => '\u{f0553}',
        Section::Themes => '\u{f03d8}',
        Section::Network => '\u{f0317}',
        Section::About => '\u{f02fd}',
    }
}
