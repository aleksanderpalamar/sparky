use std::ops::Range;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, button, column, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::system::{Control, Row};
use crate::domain::value::Step;
use crate::view::fields::{input, small};
use crate::view::style::{NUMBER_WIDTH, VALUE_WIDTH};

const ROW_PADDING: f32 = 10.0;
const CHOICE_WIDTH: f32 = 200.0;
const PAGE_ROWS: usize = 5;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let entries = settings.system_rows();
    if entries.is_empty() {
        return text("Nenhum item disponível nesta seção.").into();
    }
    let count = entries.len();
    let range = page_range(settings.focus, count);
    let start = range.start;
    let end = range.end;
    let lines = entries
        .into_iter()
        .enumerate()
        .skip(start)
        .take(PAGE_ROWS)
        .map(|(index, entry)| line(settings, index, entry));
    let rows = Column::with_children(lines).spacing(2);
    if count <= PAGE_ROWS {
        return rows.into();
    }
    let skin = settings.context.skin;
    let controls = row![
        small(skin, "◂", Message::Point(start.saturating_sub(PAGE_ROWS))),
        text(format!("{}–{} de {count}", start + 1, end)).size(TEXT_SIZE),
        small(skin, "▸", Message::Point(end.min(count - 1))),
    ].spacing(SPACING).align_y(Vertical::Center);
    column![controls, rows].spacing(SPACING).into()
}

fn page_range(focus: usize, count: usize) -> Range<usize> {
    let start = focus.min(count.saturating_sub(1)) / PAGE_ROWS * PAGE_ROWS;
    start..(start + PAGE_ROWS).min(count)
}

fn line(settings: &Settings, index: usize, entry: Row) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let highlight = Highlight::when(settings.focus == index);
    let label = text(entry.label.clone()).size(TEXT_SIZE).color(skin.bright_foreground).width(Length::Fill);
    let value = match settings.mode {
        Mode::Editing(Target::System(id)) if id == entry.id => input(settings),
        _ => control(skin, index, entry),
    };
    let content = row![label, value].spacing(SPACING).align_y(Vertical::Center);
    let cell = container(content)
        .padding([ROW_PADDING, PADDING])
        .width(Length::Fill)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Point(index)).into()
}

fn control(skin: Skin, index: usize, entry: Row) -> Element<'static, Message> {
    let value = |width: f32| text(entry.value.clone()).size(TEXT_SIZE).width(width).align_x(Horizontal::Center);
    let control: Element<'static, Message> = match entry.control {
        Control::Choice => row![
            small(skin, "◂", Message::Nudge(index, Step::Less)),
            value(CHOICE_WIDTH),
            small(skin, "▸", Message::Nudge(index, Step::More)),
        ]
        .align_y(Vertical::Center)
        .into(),
        Control::Level => row![
            small(skin, "−", Message::Nudge(index, Step::Less)),
            value(NUMBER_WIDTH),
            small(skin, "+", Message::Nudge(index, Step::More)),
        ]
        .align_y(Vertical::Center)
        .into(),
        Control::Text | Control::Toggle(_) => {
            let color = match entry.control {
                Control::Toggle(true) => skin.accent,
                _ => skin.foreground,
            };
            button(text(entry.value.clone()).size(TEXT_SIZE).color(color))
                .on_press(Message::Activate(index))
                .style(move |_, status| style::entry(skin, Highlight::Plain, status))
                .into()
        }
        Control::Reading => text(entry.value.clone()).size(TEXT_SIZE).color(skin.dark_foreground).into(),
    };
    container(control).width(VALUE_WIDTH).align_x(Horizontal::Right).into()
}

#[cfg(test)]
mod tests {
    use super::page_range;

    #[test]
    fn paginacao_mantem_a_linha_em_foco_visivel() {
        assert_eq!(page_range(0, 12), 0..5);
        assert_eq!(page_range(7, 12), 5..10);
        assert_eq!(page_range(99, 12), 10..12);
        assert_eq!(page_range(0, 0), 0..0);
    }
}
