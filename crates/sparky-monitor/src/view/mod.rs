pub mod cards;
pub mod chart;
pub mod dashboard;
pub mod details;
pub mod header;
pub mod icon;
pub mod lists;
pub mod parts;
pub mod scale;
pub mod status;
pub mod style;
pub mod table;

#[cfg(test)]
mod render_tests;

use iced::widget::{column, container};
use iced::{Element, Length};

use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    container(column![
        header::header(monitor),
        content(monitor),
        status::status(monitor)
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| sparky_ui::style::window(skin))
    .into()
}

fn content(monitor: &Monitor) -> Element<'_, Message> {
    let page = match monitor.page {
        Page::Dashboard => dashboard::dashboard(monitor),
        Page::Applications => lists::applications(monitor),
        Page::Processes => lists::processes(monitor),
        Page::Cpu => details::cpu::view(monitor),
        Page::Memory => details::memory::view(monitor),
        Page::Disk => details::disk::view(monitor),
        Page::Network => details::network::view(monitor),
        Page::Gpu => details::gpu::view(monitor),
    };
    container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn title(monitor: &Monitor) -> String {
    format!("{} — Monitor do sistema", monitor.page.label())
}
