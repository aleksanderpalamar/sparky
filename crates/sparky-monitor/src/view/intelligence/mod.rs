mod card;
mod decisions;
mod learning;
mod metrics;
mod overview;
mod rounds;
mod tabs;

use iced::Element;
use iced::widget::text;
use sparky_ui::style::{SMALL_TEXT, TEXT_SIZE};

use super::details::titled;
use super::parts::{page, waiting};
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::decision::Lens;
use crate::domain::intelligence::Advice;
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
    let body = match monitor.lens {
        Lens::Overview => overview::view(skin, briefing),
        Lens::Decisions => decisions::view(skin, &briefing.decisions),
        Lens::Learning => learning::view(skin, &briefing.status, &briefing.metrics),
    };
    let footer = text(tr("local-processing"))
        .size(SMALL_TEXT)
        .color(skin.dark_foreground);
    page(
        skin,
        tr("intelligence"),
        Some(tr("intelligence-legend")),
        vec![tabs::tabs(skin, monitor.lens), body, footer.into()],
    )
}
