use iced::widget::{Column, column, container, mouse_area, row, text};
use iced::{Alignment, Element, Length};
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::app::view::picture::{Tint, picture};
use crate::domain::app::App;
use crate::domain::icon::Icon;

pub const ROW_HEIGHT: f32 = 44.0;
pub const ROW_GAP: f32 = 2.0;
const ICON_SIZE: f32 = 28.0;
const SYMBOLIC_SIZE: f32 = 20.0;

pub fn list<'a>(launcher: &'a Launcher, results: &[&'a App]) -> Element<'a, Message> {
    let range = launcher.list.range(results.len());
    let rows = range.filter_map(|index| results.get(index).map(|app| line(launcher, app, index)));
    Column::with_children(rows).spacing(ROW_GAP).into()
}

fn line(launcher: &Launcher, app: &App, index: usize) -> Element<'static, Message> {
    let skin = launcher.skin;
    let highlight = Highlight::when(Some(index) == launcher.cursor);
    let mut lines = column![
        text(app.name.clone())
            .size(TEXT_SIZE)
            .color(skin.bright_foreground)
    ];
    if let Some(detail) = &app.detail {
        lines = lines.push(
            text(detail.clone())
                .size(SMALL_TEXT)
                .color(skin.dark_foreground),
        );
    }
    let (tint, size) = match app.icon {
        Some(Icon::Symbolic(_)) => (Tint::Symbolic, SYMBOLIC_SIZE),
        _ => (Tint::Natural, ICON_SIZE),
    };
    let icon = container(picture(skin, launcher.icons.get(&app.id), tint, size)).center(ICON_SIZE);
    let content = row![icon, lines]
        .spacing(PADDING)
        .align_y(Alignment::Center);
    let cell = container(content)
        .height(ROW_HEIGHT)
        .width(Length::Fill)
        .padding([4.0, PADDING])
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Pick(index)).into()
}
