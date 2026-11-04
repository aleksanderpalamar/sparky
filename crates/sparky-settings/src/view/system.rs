use iced::Element;
use iced::widget::{Column, column};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::section::Section;
use crate::domain::system::{Control, Row, RowId};
use crate::view::fields::sliding;
use crate::view::kit::controls::{boxed, editor, option_row, reading};
use crate::view::kit::inputs::{Level, level, picker, switch};
use crate::view::kit::rows::{Line, caption, focusable, grouped, hint};
use crate::view::kit::tokens::{CONTROL_WIDTH, space};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let rows = settings.system_rows();
    if rows.is_empty() {
        return hint(skin, "Nenhum item disponível nesta seção.");
    }
    let missing = missing(settings.section, &rows);
    let items = rows.into_iter().enumerate().map(|(index, row)| {
        (
            group(row.id),
            focusable(
                skin,
                index,
                settings.focus == index,
                line(settings, index, row),
            ),
        )
    });
    let mut body = column![grouped(skin, items)].spacing(space::MD);
    for (group, note) in missing {
        body = body.push(caption(skin, group)).push(hint(skin, note));
    }
    if let Some(note) = footnote(settings.section) {
        body = body.push(hint(skin, note));
    }
    body.into()
}

fn line(settings: &Settings, index: usize, row: Row) -> Element<'_, Message> {
    let skin = settings.context.skin;
    match &row.control {
        Control::Options(options) => {
            let choices = options.iter().map(|option| {
                option_row(
                    skin,
                    option.clone(),
                    *option == row.value,
                    Message::Choose(row.id, option.clone()),
                )
            });
            Column::with_children(choices).spacing(space::XXS).into()
        }
        _ => Line::new(row.label.clone(), control(settings, index, row)).view(skin),
    }
}

fn control(settings: &Settings, index: usize, row: Row) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if settings.mode == Mode::Editing(Target::System(row.id)) {
        return editor(skin, &settings.input, CONTROL_WIDTH);
    }
    let id = row.id;
    match (row.control, settings.pickers.get(id)) {
        (Control::Choice(_), Some(state)) => picker(skin, state, &row.value, move |value| {
            Message::Choose(id, value)
        }),
        (Control::Choice(_) | Control::Text, _) => boxed(skin, row.value, Message::Activate(index)),
        (Control::Level, _) => percent(settings, id, &row.value),
        (Control::Toggle(on), _) => switch(skin, on, Message::Activate(index)),
        (Control::Reading | Control::Options(_), _) => reading(skin, row.value),
    }
}

fn percent<'a>(settings: &Settings, id: RowId, value: &str) -> Element<'a, Message> {
    let target = Target::System(id);
    let current = value.trim_end_matches('%').parse().unwrap_or(0);
    let value = sliding(settings, target).unwrap_or(current);
    let slider = Level {
        target,
        range: 1..=100,
        step: 1,
        value,
    };
    level(settings.context.skin, slider, format!("{value}%"))
}

fn group(id: RowId) -> &'static str {
    match id {
        RowId::PowerProfile => "Perfil de energia",
        RowId::Brightness => "Tela",
        RowId::Battery => "Bateria",
        RowId::IdleLock => "Bloqueio",
        RowId::Timezone => "Fuso horário",
        RowId::LocalTime => "Relógio",
        RowId::NetworkTime | RowId::LocalClock => "Ajustes",
        RowId::Locale => "Idioma",
        RowId::ConsoleKeymap | RowId::SystemLayout => "Teclado",
        RowId::Theme | RowId::ThemeStatus => "Tema",
        RowId::Wallpaper => "Papel de parede",
        RowId::LoginWallpaper => "Tela de login",
        RowId::Volume(_) | RowId::MediaStatus => "Volumes",
    }
}

fn missing(section: Section, rows: &[Row]) -> Vec<(&'static str, &'static str)> {
    if section != Section::Power {
        return Vec::new();
    }
    let absent = |id| !rows.iter().any(|row| row.id == id);
    [
        (
            RowId::Battery,
            "Bateria",
            "Nenhuma bateria: a máquina está sempre na tomada.",
        ),
        (
            RowId::Brightness,
            "Tela",
            "Este monitor não expõe controle de brilho ao sistema.",
        ),
    ]
    .into_iter()
    .filter(|(id, _, _)| absent(*id))
    .map(|(_, group, note)| (group, note))
    .collect()
}

fn footnote(section: Section) -> Option<&'static str> {
    match section {
        Section::Region => {
            Some("Alterar idioma ou teclado pede a sua senha e vale a partir do próximo login.")
        }
        Section::DateTime => {
            Some("O relógio do hardware em hora local só é necessário para conviver com o Windows.")
        }
        Section::Themes => {
            Some("Trocar de tema muda na hora as cores de todos os aplicativos do Sparky.")
        }
        _ => None,
    }
}
