use iced::Element;
use sparky_resources::application::Progress;

use crate::app::message::Message;
use crate::domain::intelligence::{clock, errors, fraction, key};
use crate::i18n::text as tr;
use crate::view::parts::stats;
use sparky_theme::Skin;

pub fn summary<'a>(skin: Skin, progress: &Progress) -> Element<'a, Message> {
    let last = progress.last.map_or_else(
        || tr("no-data"),
        |round| {
            format!(
                "{} · {}",
                clock(round.at_seconds),
                tr(&key("outcome", &round.outcome))
            )
        },
    );
    let failures = progress
        .last
        .map_or_else(|| tr("no-data"), |round| errors(&round));
    stats(
        skin,
        vec![
            ("collected", fraction(progress.collected, progress.needed)),
            ("epoch", fraction(progress.epoch, progress.epochs)),
            ("rounds", progress.rounds.to_string()),
            ("last-round", last),
            ("round-errors", failures),
        ],
    )
}
