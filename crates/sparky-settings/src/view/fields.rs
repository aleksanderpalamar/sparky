use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, button, column, container, mouse_area, row, text, text_input};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::field::{Field, Kind};
use crate::domain::value::{Step, Value, read, shown};
use crate::view::style::{NUMBER_WIDTH, VALUE_WIDTH};

const ROW_PADDING: f32 = 8.0;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let rows = Field::of(settings.section)
        .iter()
        .enumerate()
        .map(|(index, field)| field_row(settings, index, *field));
    Column::with_children(rows).spacing(2).into()
}

fn field_row(settings: &Settings, index: usize, field: Field) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let labels = column![
        text(field.label()).size(TEXT_SIZE).color(skin.bright_foreground),
        text(field.help()).size(SMALL_TEXT).color(skin.dark_foreground),
    ]
    .width(Length::Fill);
    let value = match settings.mode {
        Mode::Editing(Target::Field(editing)) if editing == field => input(settings),
        _ => control(skin, index, field, read(field, &settings.config)),
    };
    let highlight = Highlight::when(settings.focus == index);
    let line = row![labels, value].spacing(SPACING).align_y(Vertical::Center);
    let cell = container(line)
        .padding([ROW_PADDING, PADDING])
        .width(Length::Fill)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Point(index)).into()
}

pub fn input(settings: &Settings) -> Element<'_, Message> {
    text_input("", &settings.input)
        .id(PROMPT)
        .on_input(Message::Typed)
        .on_submit(Message::Submit)
        .size(TEXT_SIZE)
        .width(VALUE_WIDTH)
        .into()
}

fn control(skin: Skin, index: usize, field: Field, value: Value) -> Element<'static, Message> {
    let control: Element<'static, Message> = match field.kind() {
        Kind::Number(_) => row![
            small(skin, "−", Message::Nudge(index, Step::Less)),
            text(shown(&value)).size(TEXT_SIZE).width(NUMBER_WIDTH).align_x(Horizontal::Center),
            small(skin, "+", Message::Nudge(index, Step::More)),
        ]
        .align_y(Vertical::Center)
        .into(),
        Kind::Flag | Kind::Text(_) => {
            let color = match &value {
                Value::Flag(true) => skin.accent,
                Value::Text(text) if text.is_empty() => skin.dark_foreground,
                _ => skin.foreground,
            };
            button(text(shown(&value)).size(TEXT_SIZE).color(color))
                .on_press(Message::Activate(index))
                .style(move |_, status| style::entry(skin, Highlight::Plain, status))
                .into()
        }
    };
    container(control).width(VALUE_WIDTH).align_x(Horizontal::Right).into()
}

fn small(skin: Skin, label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(TEXT_SIZE).color(skin.accent))
        .on_press(message)
        .style(move |_, status| style::entry(skin, Highlight::Plain, status))
        .into()
}
