use iced::alignment::Vertical;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::grid::ViewMode;
use crate::domain::listing::Hidden;
use sparky_ui::style::{self, Highlight};
use sparky_ui::style::{HEADER_HEIGHT, PADDING, SMALL_TEXT, SPACING, TITLE_SIZE};

pub fn header(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let path = text(files.shown(&files.location))
        .size(TITLE_SIZE)
        .color(skin.bright_foreground)
        .width(Length::Fill);
    let tags = tags(files)
        .into_iter()
        .map(|tag| text(tag).size(SMALL_TEXT).color(skin.cyan).into());
    let modes = row![
        view_button(files, ViewMode::Grid, "Grade", "Grade · Ctrl+1"),
        view_button(files, ViewMode::List, "Lista", "Lista · Ctrl+2")
    ]
    .spacing(4);
    container(
        row![path]
            .extend(tags)
            .push(modes)
            .spacing(SPACING * 2.0)
            .align_y(Vertical::Center),
    )
    .height(HEADER_HEIGHT)
    .width(Length::Fill)
    .padding([0.0, PADDING])
    .align_y(Vertical::Center)
    .into()
}

fn view_button<'a>(
    files: &'a Files,
    mode: ViewMode,
    label: &'static str,
    help: &'static str,
) -> Element<'a, Message> {
    let skin = files.skin;
    let highlight = Highlight::when(files.view_mode == mode);
    let control = button(text(label).size(SMALL_TEXT))
        .padding([6, 8])
        .on_press(Message::View(mode))
        .style(move |_, status| style::entry(skin, highlight, status));
    tooltip(
        control,
        text(help).size(SMALL_TEXT),
        tooltip::Position::Bottom,
    )
    .style(move |_| style::tooltip(skin))
    .into()
}

fn tags(files: &Files) -> Vec<String> {
    let filter = files.listing.filter();
    let mut tags = Vec::new();
    if !filter.is_empty() {
        tags.push(format!("filtro: {filter}"));
    }
    if files.listing.hidden() == Hidden::Shown {
        tags.push("ocultos visíveis".to_owned());
    }
    tags
}
