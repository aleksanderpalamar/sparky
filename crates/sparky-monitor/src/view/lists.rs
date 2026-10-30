use iced::Element;
use iced::widget::column;
use sparky_ui::style::{PADDING, SPACING};

use super::parts::{panel, section};
use super::style::table_rows;
use super::table;
use crate::app::message::Message;
use crate::app::state::Monitor;

pub fn applications(monitor: &Monitor) -> Element<'_, Message> {
    let rows = table_rows(monitor.size.height);
    let content = column![
        section(monitor.skin, "Aplicativos"),
        table::applications(monitor, rows)
    ]
    .spacing(SPACING);
    column![panel(monitor.skin, content)]
        .padding(PADDING)
        .into()
}

pub fn processes(monitor: &Monitor) -> Element<'_, Message> {
    let rows = table_rows(monitor.size.height);
    let content = column![
        section(monitor.skin, "Processos"),
        table::processes(monitor, rows)
    ]
    .spacing(SPACING);
    column![panel(monitor.skin, content)]
        .padding(PADDING)
        .into()
}
