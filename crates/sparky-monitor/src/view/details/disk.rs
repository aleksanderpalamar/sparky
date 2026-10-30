use iced::widget::{column, container, row};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::SPACING;

use super::cpu::dash;
use super::{titled, top_processes};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::histories::Pair;
use crate::domain::measure::DiskReading;
use crate::domain::units::{bytes, celsius, percent, rate};
use crate::view::chart::{Labels, chart};
use crate::view::parts::{page, panel, section, stats, waiting};
use crate::view::scale::Scale;
use crate::view::style::DETAIL_CHART;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(reading) = &monitor.reading else {
        return waiting(skin);
    };
    let capacity: u64 = reading.disks.iter().map(|disk| disk.sample.capacity).sum();
    let used: u64 = reading.disks.iter().map(|disk| disk.sample.used).sum();
    let read: f64 = reading.disks.iter().map(|disk| disk.read).sum();
    let write: f64 = reading.disks.iter().map(|disk| disk.write).sum();
    let numbers = stats(
        skin,
        vec![
            ("Capacidade", bytes(capacity)),
            ("Em uso", in_use(used, capacity)),
            ("Lendo", rate(read)),
            ("Escrevendo", rate(write)),
        ],
    );
    let total = &monitor.histories.disk;
    let graph = chart(
        skin,
        vec![(&total.first, skin.accent), (&total.second, skin.cyan)],
        Scale::Rate,
        Labels::Shown,
        DETAIL_CHART,
    );
    let mut content = vec![titled(
        skin,
        "Todos os discos",
        column![numbers, graph].spacing(SPACING).into(),
    )];
    for disk in &reading.disks {
        let history = monitor.histories.disks.get(&disk.sample.counters.name);
        content.push(device(skin, disk, history));
    }
    content.push(top_processes(monitor));
    page(
        skin,
        "Disco".to_owned(),
        Some("leitura em destaque, escrita em ciano".to_owned()),
        content,
    )
}

fn device<'a>(
    skin: Skin,
    disk: &'a DiskReading,
    history: Option<&'a Pair>,
) -> Element<'a, Message> {
    let sample = &disk.sample;
    let numbers = stats(
        skin,
        vec![
            (
                "Montagem",
                if sample.mounts.is_empty() {
                    dash()
                } else {
                    sample.mounts.join(", ")
                },
            ),
            ("Capacidade", bytes(sample.capacity)),
            ("Em uso", in_use(sample.used, sample.capacity)),
            ("Lendo", rate(disk.read)),
            ("Escrevendo", rate(disk.write)),
            (
                "Temperatura",
                sample.temperature.map(celsius).unwrap_or_else(dash),
            ),
        ],
    );
    let mut body = column![numbers].spacing(SPACING);
    if let Some(pair) = history {
        let reading = chart(
            skin,
            vec![(&pair.first, skin.accent)],
            Scale::Rate,
            Labels::Shown,
            DETAIL_CHART,
        );
        let writing = chart(
            skin,
            vec![(&pair.second, skin.cyan)],
            Scale::Rate,
            Labels::Shown,
            DETAIL_CHART,
        );
        body = body.push(
            row![
                container(titled(skin, "Leitura", reading)).width(Length::Fill),
                container(titled(skin, "Escrita", writing)).width(Length::Fill),
            ]
            .spacing(SPACING),
        );
    }
    let title = match &sample.model {
        Some(model) => format!("{model} ({})", sample.counters.name),
        None => sample.counters.name.clone(),
    };
    panel(skin, column![section(skin, title), body].spacing(SPACING))
}

fn in_use(used: u64, capacity: u64) -> String {
    let share = match capacity {
        0 => 0.0,
        capacity => (used as f64 / capacity as f64 * 100.0) as f32,
    };
    format!("{} ({})", bytes(used), percent(share))
}
