use iced::widget::{Column, Row, column, row, text};
use iced::{Color, Element, Length};
use sparky_resources::application::{Insight, Progress};
use sparky_resources::protocol::Status;
use sparky_theme::Skin;
use sparky_ui::style::{SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use super::card::card;
use super::{metrics, rounds};
use crate::app::message::Message;
use crate::domain::intelligence::{clock, errors, fraction, key};
use crate::i18n::text as tr;

const CHART: f32 = 180.0;
const ARCHITECTURE: &str = "GRU";

pub fn view<'a>(skin: Skin, status: &Status, metrics: &[Insight]) -> Element<'a, Message> {
    let progress = &status.progress;
    column![
        heading(skin, "neural-learning"),
        card(skin, facts(skin, status, progress)),
        heading(skin, "error-history"),
        card(skin, history(skin, progress)),
        heading(skin, "predictions"),
        card(skin, metrics::table(skin, metrics)),
    ]
    .spacing(SPACING)
    .into()
}

fn heading<'a>(skin: Skin, label: &str) -> Element<'a, Message> {
    text(tr(label))
        .size(TITLE_SIZE)
        .color(skin.bright_foreground)
        .into()
}

fn facts<'a>(skin: Skin, status: &Status, progress: &Progress) -> Element<'a, Message> {
    let missing = tr("no-data");
    let last = progress.last().map_or_else(
        || missing.clone(),
        |round| {
            format!(
                "{} · {}",
                clock(round.at_seconds),
                tr(&key("outcome", &round.outcome))
            )
        },
    );
    let failures = progress.last().map_or(missing, errors);
    let pairs = [
        ("architecture", ARCHITECTURE.to_owned()),
        ("training", tr(&key("training", &status.training))),
        ("parameters", progress.parameters.to_string()),
        ("rounds", progress.rounds.to_string()),
        ("collected", fraction(progress.collected, progress.needed)),
        ("epoch", fraction(progress.epoch, progress.epochs)),
        ("last-round", last),
        ("round-errors", failures),
    ];
    let lines = pairs.chunks(4).map(|chunk| {
        let cells = chunk.iter().map(|(label, value)| {
            column![
                text(tr(label)).size(SMALL_TEXT).color(skin.dark_foreground),
                text(value.clone())
                    .size(TEXT_SIZE)
                    .color(skin.bright_foreground),
            ]
            .spacing(2)
            .width(Length::Fill)
            .into()
        });
        Row::with_children(cells).spacing(SPACING).into()
    });
    Column::with_children(lines).spacing(SPACING * 1.5).into()
}

fn history<'a>(skin: Skin, progress: &Progress) -> Element<'a, Message> {
    if progress.history.is_empty() {
        return text(tr("no-rounds"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground)
            .into();
    }
    column![
        row![
            legend(skin.accent, tr("legend-neural")),
            legend(skin.cyan, tr("legend-statistical")),
        ]
        .spacing(SPACING * 2.0),
        rounds::chart(skin, &progress.history, CHART),
    ]
    .spacing(SPACING)
    .into()
}

fn legend<'a>(color: Color, label: String) -> Element<'a, Message> {
    row![
        text("\u{f0765}").size(SMALL_TEXT).color(color),
        text(label).size(SMALL_TEXT).color(color),
    ]
    .spacing(6)
    .into()
}
