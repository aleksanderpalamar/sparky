use iced::alignment::Vertical;
use iced::widget::{Column, Space, column, container, row, text};
use iced::{Background, Element, Length};
use sparky_resources::domain::Metric;
use sparky_theme::Skin;
use sparky_ui::style::{SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use super::card::{card, tile};
use crate::app::message::Message;
use crate::domain::decision::regressions;
use crate::domain::intelligence::{Briefing, key, percent};
use crate::domain::units::bytes;
use crate::i18n::text as tr;
use crate::view::style::SECTION_TITLE;

const SHIELD: char = '\u{f0565}';
const APP: char = '\u{f0379}';

pub fn view<'a>(skin: Skin, briefing: &Briefing) -> Element<'a, Message> {
    column![
        context(skin, briefing),
        tiles(skin, briefing),
        protected(skin, &briefing.status.guarded),
    ]
    .spacing(SPACING * 1.5)
    .into()
}

fn context<'a>(skin: Skin, briefing: &Briefing) -> Element<'a, Message> {
    let status = &briefing.status;
    let activities: Vec<String> = status
        .activities
        .iter()
        .map(|activity| tr(&key("activity", &activity)))
        .collect();
    let scene = match activities.is_empty() {
        true => tr("no-data"),
        false => activities.join(" + "),
    };
    let badge = container(
        text(tr(&key("state", &status.state)))
            .size(SMALL_TEXT)
            .color(skin.accent),
    )
    .padding([2.0, SPACING])
    .style(move |_| container::Style {
        background: Some(Background::Color(skin.accent_fill(0.14))),
        ..container::Style::default()
    });
    let mode = format!(
        "{} · {}",
        tr("service-mode"),
        tr(&key("mode", &status.mode))
    );
    row![
        column![
            text(tr("current-context"))
                .size(SMALL_TEXT)
                .color(skin.dark_foreground),
            text(scene)
                .size(SECTION_TITLE + 2.0)
                .color(skin.bright_foreground),
            text(mode).size(SMALL_TEXT).color(skin.dark_foreground),
        ]
        .spacing(4)
        .width(Length::Fill),
        badge,
    ]
    .align_y(Vertical::Top)
    .into()
}

fn tiles<'a>(skin: Skin, briefing: &Briefing) -> Element<'a, Message> {
    let status = &briefing.status;
    let missing = tr("no-data");
    let footprint = status.footprint;
    let cpu = footprint.map_or_else(
        || missing.clone(),
        |used| format!("{:.2}%", used.cpu_percent),
    );
    let ram = footprint.map_or_else(
        || missing.clone(),
        |used| match used.limit_kib {
            Some(limit) => format!("{} / {}", bytes(used.rss_kib * 1024), bytes(limit * 1024)),
            None => bytes(used.rss_kib * 1024),
        },
    );
    let pressure = briefing
        .metrics
        .iter()
        .find(|insight| insight.metric == Metric::MemoryPressure)
        .and_then(|insight| percent(insight.current))
        .unwrap_or_else(|| missing.clone());
    let active = usize::from(status.adjusting).to_string();
    let pairs = [
        ("manager-cpu", cpu),
        ("manager-ram", ram),
        ("memory-pressure", pressure),
        ("neural-training", tr(&key("training", &status.training))),
        ("active-optimizations", active),
        (
            "reverted-adjustments",
            regressions(&briefing.decisions).to_string(),
        ),
    ];
    let mut grid = Column::new().spacing(SPACING);
    for chunk in pairs.chunks(2) {
        let cells = chunk
            .iter()
            .map(|(label, value)| tile(skin, tr(label), value.clone()));
        grid = grid.push(iced::widget::Row::with_children(cells).spacing(SPACING));
    }
    grid.into()
}

fn protected<'a>(skin: Skin, names: &[String]) -> Element<'a, Message> {
    let title = text(tr("protected-apps"))
        .size(TITLE_SIZE)
        .color(skin.bright_foreground);
    if names.is_empty() {
        let empty = text(tr("no-protected"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground);
        return column![title, empty].spacing(SPACING).into();
    }
    let rows = names.iter().map(|name| {
        row![
            text(APP.to_string())
                .size(TEXT_SIZE)
                .color(skin.dark_foreground),
            text(name.clone())
                .size(TEXT_SIZE)
                .color(skin.bright_foreground),
            Space::new().width(Length::Fill),
            text(SHIELD.to_string()).size(TEXT_SIZE).color(skin.accent),
        ]
        .spacing(SPACING)
        .align_y(Vertical::Center)
        .into()
    });
    column![
        title,
        card(skin, Column::with_children(rows).spacing(SPACING))
    ]
    .spacing(SPACING)
    .into()
}
