use iced::Element;
use iced::widget::{Column, text};
use sparky_resources::domain::Change;
use sparky_resources::orchestration::Decision;
use sparky_theme::Skin;
use sparky_ui::style::TEXT_SIZE;

use crate::app::message::Message;
use crate::domain::intelligence::{clock, key};
use crate::i18n::text as tr;

pub fn list<'a>(skin: Skin, decisions: &[Decision]) -> Element<'a, Message> {
    if decisions.is_empty() {
        return text(tr("no-decisions"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground)
            .into();
    }
    let lines = decisions.iter().rev().map(|decision| {
        let line = format!(
            "{}  {}  {}  {}  {}",
            clock(decision.at_seconds),
            decision.process,
            change(&decision.change),
            tr(&key("purpose", &decision.purpose)),
            tr(&key("verdict", &decision.verdict)),
        );
        text(line)
            .size(TEXT_SIZE)
            .color(skin.bright_foreground)
            .into()
    });
    Column::with_children(lines).spacing(4).into()
}

fn change(change: &Change) -> String {
    match change {
        Change::Nice(value) => format!("nice {value}"),
        Change::IoPriority(class) => format!("ioprio {}", tr(&key("io", class))),
        Change::Affinity(cpus) => format!("cpus {cpus:?}"),
    }
}
