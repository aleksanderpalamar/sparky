use iced::widget::{column, row};
use iced::{Element, Length};
use sparky_ui::style::{PADDING, SPACING};

use super::cards::cards;
use super::parts::{link, panel, section};
use super::style::{DASHBOARD_APP_ROWS, dashboard_process_rows};
use super::table;
use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;

pub fn dashboard(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let tables = column![
        section(skin, "Aplicativos"),
        table::applications(monitor, DASHBOARD_APP_ROWS),
        link(skin, "Ver todos os aplicativos", Page::Applications),
        section(skin, "Processos"),
        table::processes(monitor, dashboard_process_rows(monitor.size.height)),
        link(skin, "Ver todos os processos", Page::Processes),
    ]
    .spacing(SPACING);
    row![
        iced::widget::container(panel(skin, tables)).width(Length::FillPortion(3)),
        iced::widget::container(cards(monitor)).width(Length::FillPortion(2)),
    ]
    .spacing(PADDING)
    .padding(PADDING)
    .height(Length::Fill)
    .into()
}
