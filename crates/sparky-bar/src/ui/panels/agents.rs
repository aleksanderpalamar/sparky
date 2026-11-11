use iced::widget::{column, progress_bar, row, rule, text};
use iced::{Alignment, Element, Length};
use sparky_theme::Skin;

use super::action;
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::agents::{Agent, AgentId, Period, Quota};
use crate::{i18n, ui};

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let summary = i18n::with(
        "agents-tokens-today",
        "tokens",
        &format_tokens(panels.agents.tokens_today()),
    );
    let heading = row![
        text(ui::agents_icon::GLYPH)
            .font(ui::style::FONT)
            .size(24)
            .color(skin.foreground),
        column![
            text(i18n::text("agents-title"))
                .size(18)
                .color(skin.foreground),
            text(summary).size(11).color(skin.dark_foreground),
        ]
        .spacing(4)
        .width(Length::Fill),
        action(skin, "+", Message::AgentsSelector),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    let mut content = column![heading, rule::horizontal(1)].spacing(14);
    if panels.agent_selector {
        content = content.push(selector(skin, panels));
    } else if panels.agents.agents.is_empty() && !panels.agent_loading {
        content = content.push(text(i18n::text("agents-empty")).color(skin.dark_foreground));
    } else {
        for agent in &panels.agents.agents {
            content = content.push(card(skin, agent)).push(rule::horizontal(1));
        }
    }
    if panels.agent_loading {
        content = content.push(
            text(i18n::text("agents-updating"))
                .size(11)
                .color(skin.accent),
        );
    }
    if let Some(error) = &panels.agents.local_error {
        content = content.push(
            text(i18n::with("agents-local-error", "reason", error))
                .size(11)
                .color(skin.urgent),
        );
    }
    if let Some(error) = &panels.agent_launch_error {
        content = content.push(text(error).size(11).color(skin.urgent));
    }
    content = content.push(action(
        skin,
        i18n::text("agents-refresh"),
        Message::AgentsRefresh,
    ));
    content.width(Length::Fill).into()
}

fn selector<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let mut choices = column![
        text(i18n::text("agents-choose"))
            .size(11)
            .color(skin.dark_foreground)
    ]
    .spacing(8);
    for agent in &panels.agents.agents {
        choices = choices.push(action(
            skin,
            agent.name.clone(),
            Message::AgentsLaunch(agent.id.clone()),
        ));
    }
    choices.into()
}

fn card<'a>(skin: &Skin, agent: &'a Agent) -> Element<'a, Message> {
    let detail = match (&agent.id, agent.tokens_today, agent.running) {
        (AgentId::Local(_), _, true) => i18n::text("agents-running"),
        (AgentId::Local(_), _, false) => i18n::text("agents-ready"),
        (_, Some(tokens), _) => i18n::with("agents-used-today", "tokens", &format_tokens(tokens)),
        _ => i18n::text("unavailable"),
    };
    let label = column![
        text(agent.name.clone()).size(14).color(skin.foreground),
        text(detail).size(11).color(skin.dark_foreground),
    ]
    .spacing(4)
    .width(Length::Fill);
    let open = action(
        skin,
        i18n::text("agents-open"),
        Message::AgentsLaunch(agent.id.clone()),
    );
    let mut controls = row![open].spacing(6).align_y(Alignment::Center);
    if let (AgentId::Local(name), true) = (&agent.id, agent.running) {
        controls = controls.push(action(
            skin,
            i18n::text("agents-stop"),
            Message::AgentsStop(name.clone()),
        ));
    }
    let mut content = column![row![label, controls].align_y(Alignment::Center)].spacing(8);
    for quota in &agent.quotas {
        content = content.push(quota_row(skin, quota));
    }
    content.into()
}

fn quota_row<'a>(skin: &Skin, quota: &Quota) -> Element<'a, Message> {
    let label = match quota.period {
        Period::FiveHours => i18n::text("agents-five-hours"),
        Period::Weekly => i18n::text("agents-weekly"),
    };
    let reset = quota
        .resets_at
        .and_then(remaining)
        .map(|time| i18n::with("agents-reset-in", "time", &time))
        .unwrap_or_default();
    let skin = *skin;
    column![
        row![
            text(label)
                .size(11)
                .color(skin.dark_foreground)
                .width(Length::Fill),
            text(format!("{}% · {reset}", quota.used_percent))
                .size(11)
                .color(skin.dark_foreground),
        ],
        progress_bar(0.0..=100.0, quota.used_percent as f32)
            .girth(4)
            .style(move |_| progress_bar::Style {
                background: skin.fill(0.16).into(),
                bar: skin.accent.into(),
                border: iced::Border::default(),
            }),
    ]
    .spacing(4)
    .into()
}

fn remaining(reset: i64) -> Option<String> {
    let seconds = reset.checked_sub(chrono::Utc::now().timestamp())?;
    if seconds <= 0 {
        return None;
    }
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    if days > 0 {
        Some(format!("{days}d {hours}h"))
    } else {
        Some(format!("{}h", hours.max(1)))
    }
}

fn format_tokens(tokens: u64) -> String {
    match tokens {
        0..=999 => tokens.to_string(),
        1_000..=999_999 => format!("{:.1}K", tokens as f64 / 1_000.0),
        _ => format!("{:.1}M", tokens as f64 / 1_000_000.0),
    }
}

#[cfg(test)]
mod tests {
    use super::format_tokens;

    #[test]
    fn resumo_formata_milhares_e_milhoes() {
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(12_300), "12.3K");
        assert_eq!(format_tokens(31_100_000), "31.1M");
    }
}
