use iced::widget::{Column, Row, column, text};
use iced::{Element, Length};
use sparky_ui::style::{SMALL_TEXT, SPACING};

use super::{titled, top_processes};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::units::{celsius, frequency, percent};
use crate::view::chart::{Labels, chart};
use crate::view::parts::{page, stats, waiting};
use crate::view::scale::Scale;
use crate::view::style::DETAIL_CHART;

const CORES_PER_ROW: usize = 4;
const CORE_CHART: f32 = 70.0;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(reading) = &monitor.reading else {
        return waiting(skin);
    };
    let info = &reading.cpu_info;
    let numbers = stats(
        skin,
        vec![
            ("Uso", percent(reading.cpu)),
            (
                "Frequência",
                info.frequency.map(frequency).unwrap_or_else(dash),
            ),
            (
                "Temperatura",
                info.temperature.map(celsius).unwrap_or_else(dash),
            ),
            ("Núcleos", reading.cores.len().to_string()),
        ],
    );
    let usage = chart(
        skin,
        vec![(&monitor.histories.cpu, skin.accent)],
        Scale::Percent,
        Labels::Shown,
        DETAIL_CHART,
    );
    let content = vec![
        titled(
            skin,
            "Uso total",
            column![numbers, usage].spacing(SPACING).into(),
        ),
        titled(skin, "Núcleos", cores(monitor, &reading.cores)),
        top_processes(monitor),
    ];
    page(skin, "CPU".to_owned(), info.model.clone(), content)
}

fn cores<'a>(monitor: &'a Monitor, usage: &[f32]) -> Element<'a, Message> {
    let skin = monitor.skin;
    let cells: Vec<Element<'a, Message>> = monitor
        .histories
        .cores
        .iter()
        .zip(usage)
        .enumerate()
        .map(|(index, (history, value))| {
            column![
                text(format!("Núcleo {index} · {}", percent(*value)))
                    .size(SMALL_TEXT)
                    .color(skin.dark_foreground),
                chart(
                    skin,
                    vec![(history, skin.accent)],
                    Scale::Percent,
                    Labels::Hidden,
                    CORE_CHART
                ),
            ]
            .spacing(4)
            .width(Length::Fill)
            .into()
        })
        .collect();
    let mut rows = Column::new().spacing(SPACING);
    let mut cells = cells.into_iter().peekable();
    while cells.peek().is_some() {
        let line: Vec<_> = cells.by_ref().take(CORES_PER_ROW).collect();
        rows = rows.push(Row::with_children(line).spacing(SPACING));
    }
    rows.into()
}

pub fn dash() -> String {
    "—".to_owned()
}
