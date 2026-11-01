use std::iter;

use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Alignment, Element, Length};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Slot, Target};
use crate::view::kit::controls::{action, editor};
use crate::view::kit::rows::{caption, cell, hint};
use crate::view::kit::tokens::{GLYPH_WIDTH, size, space};

const NOTE: &str = "Os comandos rodam na ordem da lista quando a sessão do Sparky começa.";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let entries = &settings.config.autostart;
    let rows = entries
        .iter()
        .enumerate()
        .map(|(index, command)| line(settings, index, Slot::Existing(index), command))
        .chain(iter::once(line(
            settings,
            entries.len(),
            Slot::New,
            "Adicionar comando",
        )));
    column![
        caption(skin, "Comandos ao iniciar a sessão"),
        Column::with_children(rows).spacing(space::XXS),
        hint(skin, NOTE),
    ]
    .spacing(space::MD)
    .into()
}

fn line<'a>(
    settings: &'a Settings,
    index: usize,
    slot: Slot,
    label: &'a str,
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let editing = settings.mode == Mode::Editing(Target::Autostart(slot));
    let content: Element<'a, Message> = match (editing, slot) {
        (true, _) => editor(skin, &settings.input, Length::Fill),
        (false, Slot::New) => row![
            text("+")
                .size(size::ICON)
                .width(GLYPH_WIDTH)
                .color(skin.accent),
            text(label).size(size::BODY).color(skin.accent),
        ]
        .align_y(Alignment::Center)
        .into(),
        (false, Slot::Existing(_)) => row![
            text(label).size(size::BODY).width(Length::Fill),
            action(skin, "Remover", Message::Remove(index)),
        ]
        .spacing(space::XXL)
        .align_y(Alignment::Center)
        .into(),
    };
    let padded = container(content).padding([space::MD, space::LG]).into();
    mouse_area(cell(skin, settings.focus == index, padded))
        .on_press(Message::Point(index))
        .on_double_click(Message::Activate(index))
        .into()
}
