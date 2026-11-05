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
            ("usage", percent(reading.cpu)),
            (
                "frequency",
                info.frequency.map(frequency).unwrap_or_else(dash),
            ),
            (
                "temperature",
                info.temperature.map(celsius).unwrap_or_else(dash),
            ),
            ("cores", reading.cores.len().to_string()),
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
            crate::i18n::text("total-usage"),
            column![numbers, usage].spacing(SPACING).into(),
        ),
        titled(
            skin,
            crate::i18n::text("cores"),
            cores(monitor, &reading.cores),
        ),
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
                text(core_label(index, *value))
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

fn core_label(index: usize, value: f32) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("index", index as i64);
    args.set("usage", percent(value));
    crate::i18n::message("core-reading", Some(&args))
}
