use iced::alignment::Horizontal;
use iced::widget::text::Wrapping;
use iced::widget::{Column, Row, Space, column, container, mouse_area, text, tooltip};
use iced::{Element, Length};
use sparky_ui::style::{self, Highlight, SPACING};

use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::app::view::picture::{Tint, picture};
use crate::domain::app::App;
use crate::domain::caption::caption;
use crate::domain::grid::rows;
use crate::domain::icon::ICON_PIXELS;

const CELL_HEIGHT: f32 = 108.0;
pub const ROW_STEP: f32 = CELL_HEIGHT + SPACING;
const LABEL_CHARS: usize = 14;
const LABEL_LINES: usize = 2;
const LABEL_HEIGHT: f32 = 34.0;
const LABEL_SIZE: f32 = 13.0;
const ICON_SIZE: f32 = ICON_PIXELS as f32;

pub fn grid<'a>(launcher: &'a Launcher, results: &[&'a App]) -> Element<'a, Message> {
    let columns = launcher.mode.columns();
    let lines = launcher
        .list
        .range(rows(results.len(), columns))
        .map(|line| {
            let cells =
                (line * columns..(line + 1) * columns).map(|index| match results.get(index) {
                    Some(app) => cell(launcher, app, index),
                    None => Space::new().width(Length::FillPortion(1)).into(),
                });
            Row::with_children(cells)
                .spacing(SPACING)
                .width(Length::Fill)
                .into()
        });
    let body = Column::with_children(lines)
        .spacing(SPACING)
        .width(Length::Fill)
        .height(Length::Fill);
    mouse_area(body).on_scroll(Message::Scrolled).into()
}

fn cell(launcher: &Launcher, app: &App, index: usize) -> Element<'static, Message> {
    let skin = launcher.skin;
    let highlight = Highlight::when(index == launcher.cursor);
    let shown = caption(&app.name, LABEL_CHARS, LABEL_LINES);
    let label = text(shown.text)
        .size(LABEL_SIZE)
        .color(skin.bright_foreground)
        .width(Length::Fill)
        .height(LABEL_HEIGHT)
        .align_x(Horizontal::Center)
        .wrapping(Wrapping::WordOrGlyph);
    let icon = picture(skin, launcher.icons.get(&app.id), Tint::Natural, ICON_SIZE);
    let body = column![icon, label]
        .align_x(Horizontal::Center)
        .spacing(6.0)
        .width(Length::Fill);
    let tile = container(body)
        .padding([10.0, 4.0])
        .width(Length::FillPortion(1))
        .height(CELL_HEIGHT)
        .clip(true)
        .style(move |_| style::row(skin, highlight));
    let area = mouse_area(tile)
        .on_enter(Message::Hover(index))
        .on_press(Message::Pick(index))
        .on_right_press(Message::AppMenu(app.id.clone()));
    if !shown.shortened {
        return area.into();
    }
    let full = container(text(app.name.clone()).size(LABEL_SIZE)).padding([4.0, 8.0]);
    tooltip(area, full, tooltip::Position::Bottom)
        .style(move |_| style::tooltip(skin))
        .into()
}
