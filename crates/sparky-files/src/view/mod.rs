pub mod header;
pub mod grid;
pub mod icon;
pub mod labels;
pub mod list;
pub mod sidebar;
pub mod status;
pub mod style;

use iced::widget::{column, container, row};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::grid::ViewMode;

pub fn view(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let content = match files.view_mode {
        ViewMode::Grid => grid::grid(files),
        ViewMode::List => list::list(files),
    };
    let main = column![header::header(files), content].width(Length::Fill).height(Length::Fill);
    let body = row![sidebar::sidebar(files), main].height(Length::Fill);
    container(column![body, status::status(files)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| sparky_ui::style::window(skin))
        .into()
}

pub fn title(files: &Files) -> String {
    format!("{} — Arquivos", files.shown(&files.location))
}
