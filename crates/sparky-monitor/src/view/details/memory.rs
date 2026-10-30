use iced::widget::{column, row};
use iced::{Element, Length};
use sparky_ui::style::SPACING;

use super::cpu::dash;
use super::{titled, top_processes};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::units::{bytes, percent};
use crate::view::chart::{Labels, chart};
use crate::view::parts::{page, stats, waiting};
use crate::view::scale::Scale;
use crate::view::style::DETAIL_CHART;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(reading) = &monitor.reading else {
        return waiting(skin);
    };
    let Some(memory) = reading.memory else {
        return page(
            skin,
            "Memória".to_owned(),
            Some(dash()),
            vec![top_processes(monitor)],
        );
    };
    let numbers = stats(
        skin,
        vec![
            (
                "Em uso",
                format!("{} ({})", bytes(memory.used()), percent(memory.percent())),
            ),
            ("Disponível", bytes(memory.available)),
            ("Total", bytes(memory.total)),
            (
                "Swap em uso",
                format!(
                    "{} de {}",
                    bytes(memory.swap_used()),
                    bytes(memory.swap_total)
                ),
            ),
        ],
    );
    let ram = chart(
        skin,
        vec![(&monitor.histories.memory, skin.accent)],
        Scale::Percent,
        Labels::Shown,
        DETAIL_CHART,
    );
    let swap = chart(
        skin,
        vec![(&monitor.histories.swap, skin.cyan)],
        Scale::Percent,
        Labels::Shown,
        DETAIL_CHART,
    );
    let charts = row![
        iced::widget::container(titled(skin, "Memória", ram)).width(Length::Fill),
        iced::widget::container(titled(skin, "Swap", swap)).width(Length::Fill),
    ]
    .spacing(SPACING);
    let content = vec![
        column![numbers].into(),
        charts.into(),
        top_processes(monitor),
    ];
    page(skin, "Memória".to_owned(), None, content)
}
