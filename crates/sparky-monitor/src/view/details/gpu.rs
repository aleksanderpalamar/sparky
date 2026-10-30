use iced::widget::{column, container, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{SPACING, TEXT_SIZE};

use super::cpu::dash;
use super::{titled, top_processes};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::histories::Pair;
use crate::domain::snapshot::GpuDevice;
use crate::domain::units::{bytes, percent};
use crate::view::chart::{Labels, chart};
use crate::view::parts::{page, panel, section, stats, waiting};
use crate::view::scale::Scale;
use crate::view::style::DETAIL_CHART;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(reading) = &monitor.reading else {
        return waiting(skin);
    };
    if reading.gpus.is_empty() {
        let none = text("Nenhuma GPU encontrada em /sys/class/drm")
            .size(TEXT_SIZE)
            .color(skin.dark_foreground);
        return page(skin, "GPU".to_owned(), None, vec![panel(skin, none)]);
    }
    let mut content: Vec<Element<'_, Message>> = reading
        .gpus
        .iter()
        .zip(&monitor.histories.gpus)
        .enumerate()
        .map(|(index, (gpu, pair))| device(skin, index, gpu, pair))
        .collect();
    content.push(top_processes(monitor));
    page(skin, "GPU".to_owned(), None, content)
}

fn device<'a>(skin: Skin, index: usize, gpu: &GpuDevice, pair: &'a Pair) -> Element<'a, Message> {
    let vram = match (gpu.vram_used, gpu.vram_total) {
        (Some(used), Some(total)) => format!("{} de {}", bytes(used), bytes(total)),
        (Some(used), None) => bytes(used),
        _ => dash(),
    };
    let numbers = stats(
        skin,
        vec![
            ("Uso", gpu.busy.map(percent).unwrap_or_else(dash)),
            ("VRAM em uso", vram),
        ],
    );
    let usage = chart(
        skin,
        vec![(&pair.first, skin.accent)],
        Scale::Percent,
        Labels::Shown,
        DETAIL_CHART,
    );
    let memory = chart(
        skin,
        vec![(&pair.second, skin.cyan)],
        Scale::Percent,
        Labels::Shown,
        DETAIL_CHART,
    );
    let charts = row![
        container(titled(skin, "Uso", usage)).width(Length::Fill),
        container(titled(skin, "VRAM", memory)).width(Length::Fill),
    ]
    .spacing(SPACING);
    let title = format!("GPU {index} · {}", gpu.name);
    panel(
        skin,
        column![section(skin, title), numbers, charts].spacing(SPACING),
    )
}
