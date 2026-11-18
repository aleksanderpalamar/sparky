mod decisions;
mod metrics;

use iced::Element;
use iced::widget::text;
use sparky_resources::protocol::Status;
use sparky_ui::style::TEXT_SIZE;

use super::details::titled;
use super::parts::{page, stats, waiting};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::intelligence::{Advice, key};
use crate::i18n::text as tr;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(advice) = &monitor.advice else {
        return waiting(skin);
    };
    let Advice::Ready(briefing) = advice else {
        let notice = text(tr("service-unavailable"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground);
        return page(
            skin,
            tr("intelligence"),
            None,
            vec![titled(skin, tr("overview"), notice.into())],
        );
    };
    let content = vec![
        titled(skin, tr("overview"), overview(monitor, &briefing.status)),
        titled(
            skin,
            tr("predictions"),
            metrics::table(skin, &briefing.metrics),
        ),
        titled(
            skin,
            tr("decisions"),
            decisions::list(skin, &briefing.decisions),
        ),
    ];
    page(
        skin,
        tr("intelligence"),
        Some(tr("intelligence-legend")),
        content,
    )
}

fn overview<'a>(monitor: &Monitor, status: &Status) -> Element<'a, Message> {
    let activities: Vec<String> = status
        .activities
        .iter()
        .map(|activity| tr(&key("activity", &activity)))
        .collect();
    stats(
        monitor.skin,
        vec![
            ("service-state", tr(&key("state", &status.state))),
            ("service-mode", tr(&key("mode", &status.mode))),
            ("training", tr(&key("training", &status.training))),
            (
                "activities",
                if activities.is_empty() {
                    tr("no-data")
                } else {
                    activities.join(", ")
                },
            ),
        ],
    )
}
