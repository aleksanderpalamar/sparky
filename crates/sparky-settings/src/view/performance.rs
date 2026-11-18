use iced::widget::{Column, Space, column, row};
use iced::{Element, Length};
use sparky_resources::protocol::Status;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::performance::{Knob, MODES, mode_key, tag};
use crate::view::kit::controls::{action, option_row, reading};
use crate::view::kit::inputs::switch;
use crate::view::kit::rows::{Line, caption, hint};
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let resources = &settings.config.resources;
    let mut content = column![
        Line::new(
            text("performance-enabled"),
            switch(
                skin,
                resources.enabled,
                Message::PerformanceToggle(Knob::Enabled)
            )
        )
        .note(text("performance-enabled-note"))
        .view(skin),
        Line::new(
            text("performance-learning"),
            switch(
                skin,
                resources.learning,
                Message::PerformanceToggle(Knob::Learning)
            )
        )
        .note(text("performance-learning-note"))
        .view(skin),
        caption(skin, &text("performance-mode")),
        modes(settings),
        caption(skin, &text("performance-status")),
        status(settings),
    ]
    .spacing(space::MD);
    content = content.push(forget(settings));
    content.into()
}

fn modes(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let options = MODES.into_iter().map(|mode| {
        let label = text(&format!("performance-mode-{}", mode_key(mode)));
        option_row(
            skin,
            label,
            settings.config.resources.mode == mode,
            Message::PerformanceMode(mode),
        )
    });
    Column::with_children(options).spacing(space::XXS).into()
}

fn status(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let Some(status) = &settings.performance.status else {
        let reason = settings.performance.error.clone().unwrap_or_default();
        return hint(
            skin,
            crate::i18n::with("performance-unreachable", "reason", &reason),
        );
    };
    let lines = rows(status)
        .into_iter()
        .map(|(label, value)| Line::new(text(label), reading(skin, value)).view(skin));
    Column::with_children(lines).spacing(space::XXS).into()
}

fn rows(status: &Status) -> Vec<(&'static str, String)> {
    let named = |prefix: &str, key: String| text(&format!("{prefix}-{key}"));
    let activities: Vec<String> = status
        .activities
        .iter()
        .map(|activity| named("performance-activity", tag(&activity, "activity")))
        .collect();
    vec![
        (
            "performance-state",
            named("performance-state", tag(&status.state, "state")),
        ),
        (
            "performance-training",
            named("performance-training", tag(&status.training, "training")),
        ),
        ("performance-activities", activities.join(", ")),
    ]
}

fn forget(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if !settings.performance.confirming {
        return row![
            Space::new().width(Length::Fill),
            action(
                skin,
                text("performance-forget"),
                Message::PerformanceForget(false)
            )
        ]
        .into();
    }
    column![
        hint(skin, text("performance-forget-confirm")),
        row![
            Space::new().width(Length::Fill),
            action(skin, text("performance-cancel"), Message::PerformanceCancel),
            action(
                skin,
                text("performance-forget-yes"),
                Message::PerformanceForget(true)
            ),
        ]
        .spacing(space::MD),
    ]
    .spacing(space::MD)
    .into()
}

fn text(key: &str) -> String {
    crate::i18n::text(key)
}
