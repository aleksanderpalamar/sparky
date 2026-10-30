use iced::alignment::Vertical;
use iced::widget::{container, row, text};
use iced::{Element, Length};
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, STATUS_HEIGHT, TEXT_SIZE};

use crate::app::actions::question;
use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;

const TABLE_HINTS: &str =
    "j/k escolhe · c m g v d n ordena · Del encerra · Shift+Del força · Esc volta";
const DASHBOARD_HINTS: &str =
    "1–8 páginas · Tab troca a tabela · j/k escolhe · Del encerra · q fecha";
const READ_ONLY_HINTS: &str = "1–8 páginas · Esc volta · q fecha";

pub fn status(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let left = match (&monitor.confirm, &monitor.notice) {
        (Some(confirm), _) => text(question(confirm)).color(skin.urgent),
        (None, Some(notice)) => text(notice.text().to_owned()).color(notice.color(skin)),
        (None, None) => text(summary(monitor)).color(skin.foreground),
    };
    let line = row![
        left.size(TEXT_SIZE).width(Length::Fill),
        text(hints(monitor.page))
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
        None => "Lendo o sistema…".to_owned(),
        Some(_) => format!(
            "{} processos · {} aplicativos",
            monitor.processes.len(),
            monitor.apps.len()
        ),
    }
}

fn hints(page: Page) -> &'static str {
    match page {
        Page::Dashboard => DASHBOARD_HINTS,
        Page::Network => READ_ONLY_HINTS,
        Page::Applications
        | Page::Processes
        | Page::Cpu
        | Page::Memory
        | Page::Disk
        | Page::Gpu => TABLE_HINTS,
    }
}
