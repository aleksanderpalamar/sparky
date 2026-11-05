use fluent_bundle::FluentArgs;
use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

use crate::app::actions::question;
use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;

pub fn status(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let left = match (&monitor.confirm, &monitor.notice) {
        (Some(confirm), _) => text(question(confirm)).color(skin.urgent),
        (None, Some(notice)) => text(notice.text().to_owned()).color(notice.color(skin)),
        (None, None) => text(summary(monitor)).color(skin.foreground),
    };
    let line = row![
        left.size(TEXT_SIZE).width(Length::Fill),
        text(crate::i18n::text(hints(monitor.page)))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(line)
        .height(STATUS_HEIGHT)
        .width(Length::Fill)
        .padding([0.0, PADDING])
        .align_y(Vertical::Center)
        .style(move |_| style::panel(skin))
        .into()
}

pub fn summary(monitor: &Monitor) -> String {
    match monitor.reading {
        None => crate::i18n::text("reading"),
        Some(_) => {
            let mut args = FluentArgs::new();
            args.set("processes", monitor.processes.len().to_string());
            args.set("applications", monitor.apps.len().to_string());
            crate::i18n::message("summary", Some(&args))
        }
    }
}

fn hints(page: Page) -> &'static str {
    match page {
        Page::Dashboard => "dashboard-hints",
        Page::Network => "read-only-hints",
        Page::Applications
        | Page::Processes
        | Page::Cpu
        | Page::Memory
        | Page::Disk
        | Page::Gpu => "table-hints",
    }
}
