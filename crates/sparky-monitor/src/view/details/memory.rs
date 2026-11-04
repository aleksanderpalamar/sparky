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
            crate::i18n::text("memory"),
            Some(dash()),
            vec![top_processes(monitor)],
        );
    };
    let numbers = stats(
        skin,
        vec![
            (
                "in-use",
                format!("{} ({})", bytes(memory.used()), percent(memory.percent())),
            ),
            ("available", bytes(memory.available)),
            ("total", bytes(memory.total)),
            (
                "swap-in-use",
                swap_usage(memory.swap_used(), memory.swap_total),
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
        iced::widget::container(titled(skin, crate::i18n::text("memory"), ram)).width(Length::Fill),
        iced::widget::container(titled(skin, crate::i18n::text("swap"), swap)).width(Length::Fill),
    ]
    .spacing(SPACING);
    let content = vec![
        column![numbers].into(),
        charts.into(),
        top_processes(monitor),
    ];
    page(skin, crate::i18n::text("memory"), None, content)
}

fn swap_usage(used: u64, total: u64) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("used", bytes(used));
    args.set("total", bytes(total));
    crate::i18n::message("amount-of-total", Some(&args))
}
