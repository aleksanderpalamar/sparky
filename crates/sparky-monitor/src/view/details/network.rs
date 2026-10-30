use iced::Element;
use iced::widget::column;
use sparky_theme::Skin;
use sparky_ui::style::SPACING;

use super::titled;
use crate::app::message::Message;
use crate::app::state::Monitor;
use crate::domain::histories::Pair;
use crate::domain::units::rate;
use crate::view::chart::{Labels, chart};
use crate::view::parts::{page, panel, section, stats, waiting};
use crate::view::scale::Scale;
use crate::view::style::DETAIL_CHART;

pub fn view(monitor: &Monitor) -> Element<'_, Message> {
    let skin = monitor.skin;
    let Some(reading) = &monitor.reading else {
        return waiting(skin);
    };
    let received: f64 = reading.networks.iter().map(|net| net.received).sum();
    let sent: f64 = reading.networks.iter().map(|net| net.sent).sum();
    let mut content = vec![titled(
        skin,
        "Todas as interfaces",
        traffic(skin, &monitor.histories.network, received, sent),
    )];
    for net in &reading.networks {
        if let Some(pair) = monitor.histories.networks.get(&net.name) {
            let body = traffic(skin, pair, net.received, net.sent);
            content.push(panel(
                skin,
                column![section(skin, net.name.clone()), body].spacing(SPACING),
            ));
        }
    }
    page(
        skin,
        "Rede".to_owned(),
        Some("recebido em destaque, enviado em ciano".to_owned()),
        content,
    )
}

fn traffic(skin: Skin, pair: &Pair, received: f64, sent: f64) -> Element<'_, Message> {
    let numbers = stats(
        skin,
        vec![("Recebendo", rate(received)), ("Enviando", rate(sent))],
    );
    let graph = chart(
        skin,
        vec![(&pair.first, skin.accent), (&pair.second, skin.cyan)],
        Scale::Rate,
        Labels::Shown,
        DETAIL_CHART,
    );
    column![numbers, graph].spacing(SPACING).into()
}
