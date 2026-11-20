use iced::alignment::Vertical;
use iced::widget::{Column, column, row, text};
use iced::{Element, Font, Length, font};
use sparky_resources::orchestration::{Decision, Verdict};
use sparky_theme::Skin;
use sparky_ui::style::{FONT, SMALL_TEXT, SPACING, TEXT_SIZE};

use super::card::{card, shade};
use crate::app::message::Message;
use crate::domain::decision::{title, tone};
use crate::domain::intelligence::{clock, key};
use crate::i18n::text as tr;

const SHOWN: usize = 30;
const BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..FONT
};

pub fn view<'a>(skin: Skin, decisions: &[Decision]) -> Element<'a, Message> {
    let caption = text(tr("recent-optimizations"))
        .size(SMALL_TEXT)
        .color(skin.dark_foreground);
    if decisions.is_empty() {
        let empty = text(tr("no-decisions"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground);
        return column![caption, empty].spacing(SPACING).into();
    }
    let cards = decisions
        .iter()
        .rev()
        .take(SHOWN)
        .map(|decision| entry(skin, decision));
    column![caption, Column::with_children(cards).spacing(SPACING)]
        .spacing(SPACING)
        .into()
}

fn entry<'a>(skin: Skin, decision: &Decision) -> Element<'a, Message> {
    let color = shade(skin, tone(&decision.verdict));
    let subtitle = format!(
        "{} · {} · {} · {}",
        decision.process,
        decision.change.summary(),
        tr(&key("purpose", &decision.purpose)),
        clock(decision.at_seconds),
    );
    let body = row![
        text(glyph(&decision.verdict).to_string())
            .size(TEXT_SIZE)
            .color(color),
        column![
            text(tr(title(&decision.change)))
                .size(TEXT_SIZE)
                .font(BOLD)
                .color(skin.bright_foreground),
            text(subtitle).size(SMALL_TEXT).color(skin.dark_foreground),
        ]
        .spacing(4)
        .width(Length::Fill),
        text(tr(&key("verdict", &decision.verdict)))
            .size(SMALL_TEXT)
            .color(color),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Top);
    card(skin, body)
}

fn glyph(verdict: &Verdict) -> char {
    match verdict {
        Verdict::Applied | Verdict::Kept => '\u{f05e1}',
        Verdict::Simulated => '\u{f06d0}',
        Verdict::Blocked(_) => '\u{f0499}',
        Verdict::Reverted(_) => '\u{f054c}',
        Verdict::Failed => '\u{f05d6}',
        Verdict::Finished(_) => '\u{f0667}',
    }
}
