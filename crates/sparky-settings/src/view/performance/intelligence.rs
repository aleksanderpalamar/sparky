use iced::Element;
use iced::widget::column;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::performance::Knob;
use crate::view::kit::card::{group, title};
use crate::view::kit::controls::reading;
use crate::view::kit::inputs::{locked_switch, switch};
use crate::view::kit::rows::Line;
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let learning = Line::new(
        text("performance-learning"),
        switch(
            skin,
            settings.config.resources.learning,
            Message::PerformanceToggle(Knob::Learning),
        ),
    )
    .note(text("performance-learning-note"))
    .view(skin);
    let games = Line::new(text("performance-games"), locked_switch(skin, true))
        .note(text("performance-games-note"))
        .view(skin);
    let protection = Line::new(text("performance-protection"), locked_switch(skin, true))
        .note(text("performance-protection-note"))
        .view(skin);
    let notify = Line::new(
        text("performance-notify"),
        switch(
            skin,
            settings.config.resources.notify,
            Message::PerformanceToggle(Knob::Notify),
        ),
    )
    .note(text("performance-notify-note"))
    .view(skin);
    let guarded = settings
        .performance
        .status
        .as_ref()
        .map(|status| status.guarded.join(", "))
        .filter(|names| !names.is_empty())
        .unwrap_or_else(|| text("performance-guarded-none"));
    let now = Line::new(text("performance-guarded"), reading(skin, guarded)).view(skin);
    column![
        title(skin, text("performance-intelligence")),
        group(skin, vec![learning, games, protection, notify, now]),
    ]
    .spacing(space::SM)
    .into()
}

fn text(key: &str) -> String {
    crate::i18n::text(key)
}
