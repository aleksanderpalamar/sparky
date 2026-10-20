use iced::alignment::Vertical;
use iced::widget::{Column, button, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Slot, Target};
use crate::view::fields::input;
use crate::view::style::LINE_HEIGHT;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let entries = &settings.config.autostart;
    let rows = entries
        .iter()
        .enumerate()
        .map(|(index, command)| line(settings, index, Slot::Existing(index), command))
        .chain(std::iter::once(line(settings, entries.len(), Slot::New, "+ Adicionar comando")));
    Column::with_children(rows).spacing(2).into()
}

fn line<'a>(settings: &'a Settings, index: usize, slot: Slot, label: &'a str) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let editing = settings.mode == Mode::Editing(Target::Autostart(slot));
    let content: Element<'a, Message> = match (editing, slot) {
        (true, _) => input(settings),
        (false, Slot::New) => text(label).size(TEXT_SIZE).color(skin.accent).into(),
        (false, Slot::Existing(_)) => row![
            text(label).size(TEXT_SIZE).width(Length::Fill),
            button(text("remover").size(SMALL_TEXT).color(skin.urgent))
                .on_press(Message::Remove(index))
                .style(move |_, status| style::entry(skin, Highlight::Plain, status)),
        ]
        .spacing(SPACING)
        .align_y(Vertical::Center)
        .into(),
    };
    let highlight = Highlight::when(settings.focus == index);
    let cell = container(content)
        .height(LINE_HEIGHT)
        .padding([0.0, PADDING])
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell)
        .on_press(Message::Point(index))
        .on_double_click(Message::Activate(index))
        .into()
}
