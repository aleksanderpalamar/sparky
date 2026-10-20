pub mod header;
pub mod labels;
pub mod list;
pub mod sidebar;
pub mod status;
pub mod style;

use iced::widget::{column, container, row};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::state::Files;

pub fn view(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let main = column![header::header(files), list::list(files)].width(Length::Fill).height(Length::Fill);
    let body = row![sidebar::sidebar(files), main].height(Length::Fill);
    container(column![body, status::status(files)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| style::window(skin))
        .into()
}

pub fn title(files: &Files) -> String {
    format!("{} — Arquivos", files.shown(&files.location))
}
