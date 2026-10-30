pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;

use iced::Element;
use iced::widget::column;
use sparky_theme::Skin;
use sparky_ui::style::SPACING;

use super::parts::{panel, section};
use super::style::DETAIL_ROWS;
use super::table;
use crate::app::message::Message;
use crate::app::state::Monitor;

pub fn top_processes(monitor: &Monitor) -> Element<'_, Message> {
    let content = column![
        section(monitor.skin, "Processos"),
        table::processes(monitor, DETAIL_ROWS)
    ]
    .spacing(SPACING);
    panel(monitor.skin, content)
}

pub fn titled<'a>(
    skin: Skin,
    title: &'a str,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    panel(
        skin,
        column![section(skin, title), content].spacing(SPACING),
    )
}
