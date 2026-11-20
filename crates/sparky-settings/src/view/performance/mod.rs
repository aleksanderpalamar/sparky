mod advanced;
mod intelligence;
mod manager;

use iced::widget::{column, row, text};
use iced::{Alignment, Element};
use sparky_resources::domain::ServiceState;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::performance::tag;
use crate::view::kit::card::heading;
use crate::view::kit::tokens::{size, space};

const DOT: char = '\u{f0765}';

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    column![
        heading(
            skin,
            text_of("section-performance-label"),
            text_of("section-performance-summary")
        ),
        manager::view(settings),
        intelligence::view(settings),
        advanced::view(settings),
        service(settings),
    ]
    .spacing(space::XL)
    .into()
}

fn service(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let (color, label) = match &settings.performance.status {
        None => (
            skin.urgent,
            crate::i18n::with(
                "performance-unreachable",
                "reason",
                settings.performance.error.as_deref().unwrap_or_default(),
            ),
        ),
        Some(status) => {
            let state = text_of(&format!(
                "performance-state-{}",
                tag(&status.state, "state")
            ));
            let key = match status.state {
                ServiceState::Disabled => "performance-service-off",
                ServiceState::Safe | ServiceState::Degraded => "performance-service-attention",
                _ => "performance-service-on",
            };
            let color = match status.state {
                ServiceState::Disabled => skin.dark_foreground,
                ServiceState::Safe | ServiceState::Degraded => skin.urgent,
                _ => skin.accent,
            };
            (color, crate::i18n::with(key, "state", &state))
        }
    };
    row![
        text(DOT.to_string()).size(size::SMALL).color(color),
        text(label).size(size::SMALL).color(color),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

fn text_of(key: &str) -> String {
    crate::i18n::text(key)
}
