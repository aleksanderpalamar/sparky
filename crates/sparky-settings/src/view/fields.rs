use iced::Element;

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::field::{Field, Kind};
use crate::domain::value::{Value, read, shown};
use crate::view::kit::controls::{boxed, editor};
use crate::view::kit::inputs::{Level, level, switch};
use crate::view::kit::rows::{Line, focusable, grouped};
use crate::view::kit::tokens::CONTROL_WIDTH;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let rows = Field::of(settings.section)
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let line =
                Line::new(field.label(), control(settings, index, *field)).note(field.help());
            (
                group(*field),
                focusable(skin, index, settings.focus == index, line.view(skin)),
            )
        });
    grouped(skin, rows)
}

pub fn group(field: Field) -> &'static str {
    match field {
        Field::GapsInner | Field::GapsOuter | Field::Border => "Janelas",
        Field::FloatWidth | Field::FloatHeight => "Janelas flutuantes",
        Field::DefaultBinds => "Atalhos",
        Field::Wallpaper => "Papel de parede",
        Field::LoginWallpaper => "Tela de login",
        Field::Layout | Field::Variant | Field::Options => "Layout",
        Field::RepeatRate | Field::RepeatDelay => "Repetição",
        Field::Terminal
        | Field::Launcher
        | Field::Files
        | Field::Settings
        | Field::Screenshot
        | Field::Locker
        | Field::SystemMenu => "Programas dos atalhos",
    }
}

fn control(settings: &Settings, index: usize, field: Field) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if settings.mode == Mode::Editing(Target::Field(field)) {
        return editor(skin, &settings.input, CONTROL_WIDTH);
    }
    let value = read(field, &settings.config);
    match (field.kind(), value) {
        (Kind::Number(bounds), Value::Number(number)) => {
            let target = Target::Field(field);
            let value = sliding(settings, target).unwrap_or(number);
            let range = Level {
                target,
                range: bounds.min..=bounds.max,
                step: bounds.step,
                value,
            };
            level(skin, range, value.to_string())
        }
        (_, Value::Flag(on)) => switch(skin, on, Message::Activate(index)),
        (_, value) => boxed(skin, shown(&value), Message::Activate(index)),
    }
}

pub fn sliding(settings: &Settings, target: Target) -> Option<i32> {
    settings
        .sliding
        .filter(|(sliding, _)| *sliding == target)
        .map(|(_, value)| value)
}
