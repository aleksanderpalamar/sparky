use iced::alignment::Vertical;
use iced::widget::text::Wrapping;
use iced::widget::{Column, column, container, row, scrollable, text};
use iced::{Color, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{SMALL_TEXT, SPACING, TEXT_SIZE};

use super::chart::{Labels, chart};
use super::parts::{link, panel};
use super::scale::Scale;
use super::style::{CARD_CHART, SECTION_TITLE};
use crate::app::message::Message;
use crate::app::page::Page;
use crate::app::state::Monitor;
use crate::domain::history::History;
use crate::domain::measure::Reading;
use crate::domain::table::{Column as Field, top};
use crate::domain::units::{bytes, celsius, frequency, percent, rate};

struct Card<'a> {
    title: String,
    lines: Vec<String>,
    top: Option<(String, String)>,
    series: Vec<(&'a History, Color)>,
    scale: Scale,
    page: Page,
}

pub fn cards(monitor: &Monitor) -> Element<'_, Message> {
    let Some(reading) = &monitor.reading else {
        return super::parts::waiting(monitor.skin);
    };
    let skin = monitor.skin;
    let mut all = vec![
        cpu(monitor, reading),
        memory(monitor, reading),
        disk(monitor, reading),
        network(monitor, reading),
    ];
    all.extend(gpus(monitor, reading));
    let cards = all.into_iter().map(|card| render(skin, card));
    scrollable(Column::with_children(cards).spacing(SPACING))
        .height(Length::Fill)
        .into()
}

fn cpu<'a>(monitor: &'a Monitor, reading: &Reading) -> Card<'a> {
    let info = &reading.cpu_info;
    let details = [
        Some(percent(reading.cpu)),
        info.frequency.map(frequency),
        info.temperature.map(celsius),
    ];
    Card {
        title: "CPU".to_owned(),
        lines: vec![joined(details), info.model.clone().unwrap_or_default()],
        top: top(&monitor.processes, Field::Cpu)
            .map(|row| (row.name.clone(), percent(row.usage.cpu))),
        series: vec![(&monitor.histories.cpu, monitor.skin.accent)],
        scale: Scale::Percent,
        page: Page::Cpu,
    }
}

fn memory<'a>(monitor: &'a Monitor, reading: &Reading) -> Card<'a> {
    let memory = reading.memory.unwrap_or_default();
    Card {
        title: "Memória".to_owned(),
        lines: vec![
            format!("{} · {}", percent(memory.percent()), bytes(memory.used())),
            format!("de {}", bytes(memory.total)),
        ],
        top: top(&monitor.processes, Field::Memory)
            .map(|row| (row.name.clone(), bytes(row.usage.memory))),
        series: vec![(&monitor.histories.memory, monitor.skin.accent)],
        scale: Scale::Percent,
        page: Page::Memory,
    }
}

fn disk<'a>(monitor: &'a Monitor, reading: &Reading) -> Card<'a> {
    let (read, write) = reading
        .disks
        .iter()
        .fold((0.0, 0.0), |(read, write), disk| {
            (read + disk.read, write + disk.write)
        });
    Card {
        title: "Disco".to_owned(),
        lines: vec![format!("{} lendo · {} escrevendo", rate(read), rate(write))],
        top: top(&monitor.processes, Field::Disk)
            .map(|row| (row.name.clone(), rate(row.usage.disk))),
        series: vec![
            (&monitor.histories.disk.first, monitor.skin.accent),
            (&monitor.histories.disk.second, monitor.skin.cyan),
        ],
        scale: Scale::Rate,
        page: Page::Disk,
    }
}

fn network<'a>(monitor: &'a Monitor, reading: &Reading) -> Card<'a> {
    let (received, sent) = reading
        .networks
        .iter()
        .fold((0.0, 0.0), |(received, sent), net| {
            (received + net.received, sent + net.sent)
        });
    let busiest = reading
        .networks
        .iter()
        .max_by(|a, b| (a.received + a.sent).total_cmp(&(b.received + b.sent)));
    Card {
        title: "Rede".to_owned(),
        lines: vec![format!(
            "{} recebendo · {} enviando",
            rate(received),
            rate(sent)
        )],
        top: busiest.map(|net| (net.name.clone(), rate(net.received + net.sent))),
        series: vec![
            (&monitor.histories.network.first, monitor.skin.accent),
            (&monitor.histories.network.second, monitor.skin.cyan),
        ],
        scale: Scale::Rate,
        page: Page::Network,
    }
}

fn gpus<'a>(monitor: &'a Monitor, reading: &Reading) -> Vec<Card<'a>> {
    let busiest = top(&monitor.processes, Field::Gpu).filter(|row| row.usage.gpu.is_some());
    reading
        .gpus
        .iter()
        .zip(&monitor.histories.gpus)
        .enumerate()
        .map(|(index, (gpu, history))| Card {
            title: format!("GPU {index} · {}", gpu.name),
            lines: vec![joined([
                gpu.busy.map(percent),
                gpu.vram_used.map(|used| format!("VRAM {}", bytes(used))),
            ])],
            top: busiest.map(|row| {
                (
                    row.name.clone(),
                    row.usage.gpu.map(percent).unwrap_or_default(),
                )
            }),
            series: vec![(&history.first, monitor.skin.accent)],
            scale: Scale::Percent,
            page: Page::Gpu,
        })
        .collect()
}

fn render<'a>(skin: Skin, card: Card<'a>) -> Element<'a, Message> {
    let mut info = column![
        text(card.title)
            .size(SECTION_TITLE)
            .color(skin.bright_foreground)
    ]
    .spacing(6);
    for (index, line) in card
        .lines
        .into_iter()
        .filter(|line| !line.is_empty())
        .enumerate()
    {
        let size = if index == 0 { TEXT_SIZE } else { SMALL_TEXT };
        info = info.push(text(line).size(size).color(skin.foreground));
    }
    if let Some((name, value)) = card.top {
        info = info.push(
            row![
                container(text(name).size(TEXT_SIZE).wrapping(Wrapping::None))
                    .width(Length::Fill)
                    .clip(true),
                text(value).size(TEXT_SIZE).wrapping(Wrapping::None)
            ]
            .spacing(SPACING)
            .align_y(Vertical::Center),
        );
    }
    info = info.push(link(skin, "Detalhes", card.page));
    let graph = chart(skin, card.series, card.scale, Labels::Hidden, CARD_CHART);
    panel(
        skin,
        row![
            container(graph).width(Length::FillPortion(1)),
            info.width(Length::FillPortion(1))
        ]
        .spacing(SPACING * 2.0),
    )
}

fn joined<const N: usize>(parts: [Option<String>; N]) -> String {
    parts.into_iter().flatten().collect::<Vec<_>>().join(" · ")
}
