use iced::widget::{Column, row, text};
use iced::{Element, Length};
use sparky_resources::application::Insight;
use sparky_theme::Skin;
use sparky_ui::style::{SMALL_TEXT, TEXT_SIZE};

use crate::app::message::Message;
use crate::domain::intelligence::{key, percent, score};
use crate::i18n::text as tr;

const HEADERS: [&str; 7] = [
    "metric",
    "current",
    "statistical-short",
    "statistical-long",
    "neural-short",
    "neural-long",
    "anomaly",
];

pub fn table<'a>(skin: Skin, metrics: &[Insight]) -> Element<'a, Message> {
    let header = line(
        HEADERS.iter().map(|label| tr(label)).collect(),
        SMALL_TEXT,
        skin.dark_foreground,
    );
    let rows = metrics.iter().map(|insight| {
        let missing = tr("no-data");
        let cells = vec![
            tr(&key("metric", &insight.metric)),
            percent(insight.current).unwrap_or_else(|| missing.clone()),
            percent(insight.estimate_short).unwrap_or_else(|| missing.clone()),
            percent(insight.estimate_long).unwrap_or_else(|| missing.clone()),
            percent(insight.neural_short).unwrap_or_else(|| missing.clone()),
            percent(insight.neural_long).unwrap_or_else(|| missing.clone()),
            score(insight.anomaly).unwrap_or(missing),
        ];
        line(cells, TEXT_SIZE, skin.bright_foreground)
    });
    Column::with_children(std::iter::once(header).chain(rows))
        .spacing(4)
        .into()
}

fn line<'a>(cells: Vec<String>, size: f32, color: iced::Color) -> Element<'a, Message> {
    let widgets = cells.into_iter().enumerate().map(|(index, cell)| {
        let portion = if index == 0 { 3 } else { 2 };
        text(cell)
            .size(size)
            .color(color)
            .width(Length::FillPortion(portion))
            .into()
    });
    row(widgets).spacing(8).into()
}
