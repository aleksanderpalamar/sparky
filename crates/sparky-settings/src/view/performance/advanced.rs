use iced::widget::{Column, Space, column, row, text};
use iced::{Element, Length};
use sparky_resources::protocol::Status;
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::view::kit::card::{fold, group};
use crate::view::kit::controls::action;
use crate::view::kit::rows::hint;
use crate::view::kit::tokens::{size, space};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let open = settings.performance.advanced;
    let header = fold(
        skin,
        say("performance-advanced"),
        open,
        Message::PerformanceAdvanced,
    );
    if !open {
        return header;
    }
    let facts = facts(skin, settings.performance.status.as_ref());
    column![header, group(skin, vec![facts, forget(settings)])]
        .spacing(space::SM)
        .into()
}

fn facts<'a>(skin: Skin, status: Option<&Status>) -> Element<'a, Message> {
    let missing = say("performance-unknown");
    let limit = status
        .and_then(|status| status.footprint)
        .and_then(|footprint| footprint.limit_kib)
        .map_or_else(|| missing.clone(), |kib| format!("{} MiB", kib / 1024));
    let parameters = status.map_or(0, |status| status.progress.parameters);
    let model = match parameters {
        0 => "GRU".to_owned(),
        count => crate::i18n::with("performance-model-value", "count", &count.to_string()),
    };
    let pairs = [
        ("performance-memory-limit", limit, skin.bright_foreground),
        (
            "performance-gpu",
            say("performance-gpu-none"),
            skin.bright_foreground,
        ),
        ("performance-model", model, skin.bright_foreground),
        (
            "performance-rollback",
            say("performance-rollback-on"),
            skin.accent,
        ),
        (
            "performance-storage",
            say("performance-storage-local"),
            skin.bright_foreground,
        ),
    ];
    let lines = pairs.into_iter().map(|(label, value, color)| {
        row![
            text(say(label))
                .size(size::BODY)
                .color(skin.dark_foreground)
                .width(Length::Fill),
            text(value).size(size::BODY).color(color),
        ]
        .into()
    });
    Column::with_children(lines)
        .spacing(space::LG)
        .padding([space::LG, space::LG])
        .into()
}

fn forget(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let mut body = Column::new()
        .spacing(space::MD)
        .padding([space::LG, space::LG]);
    if settings.performance.confirming {
        body = body.push(hint(skin, say("performance-forget-confirm")));
        body = body.push(
            row![
                Space::new().width(Length::Fill),
                action(skin, say("performance-cancel"), Message::PerformanceCancel),
                action(
                    skin,
                    say("performance-forget-yes"),
                    Message::PerformanceForget(true)
                ),
            ]
            .spacing(space::MD),
        );
        return body.into();
    }
    body.push(
        row![
            text(say("performance-forget-note"))
                .size(size::SMALL)
                .color(skin.dark_foreground)
                .width(Length::Fill),
            action(
                skin,
                say("performance-forget"),
                Message::PerformanceForget(false)
            ),
        ]
        .spacing(space::MD),
    )
    .into()
}

fn say(key: &str) -> String {
    crate::i18n::text(key)
}
