use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::listing::Hidden;
use crate::view::style::{HEADER_HEIGHT, PADDING, SMALL_TEXT, SPACING};

const PATH_SIZE: f32 = 16.0;

pub fn header(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let path = text(files.shown(&files.location)).size(PATH_SIZE).color(skin.bright_foreground).width(Length::Fill);
    let tags = tags(files).into_iter().map(|tag| text(tag).size(SMALL_TEXT).color(skin.cyan).into());
    container(row![path].extend(tags).spacing(SPACING * 2.0).align_y(Vertical::Center))
        .height(HEADER_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
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
