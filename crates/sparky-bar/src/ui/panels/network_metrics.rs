use iced::widget::{column, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::network::NetworkDevice;
use crate::domain::telemetry::{Metrics, Ping, bytes};

pub fn view<'a>(
    skin: &Skin,
    device: &NetworkDevice,
    metrics: Option<&Metrics>,
) -> Element<'a, Message> {
    let metrics = metrics.filter(|metrics| metrics.interface == device.interface);
    let ping = ping(metrics.and_then(|metrics| metrics.ping));
    let loss = metrics
        .and_then(|metrics| metrics.loss_percent.map(|value| format!("{value}%")))
        .unwrap_or_else(|| "—".to_owned());
    let received = metrics.map_or_else(
        || "—".to_owned(),
        |metrics| format!("{}/s", bytes(metrics.receive_rate)),
    );
    let sent = metrics.map_or_else(
        || "—".to_owned(),
        |metrics| format!("{}/s", bytes(metrics.send_rate)),
    );
    let total_received = metrics.map_or_else(|| "—".to_owned(), |metrics| bytes(metrics.received));
    let total_sent = metrics.map_or_else(|| "—".to_owned(), |metrics| bytes(metrics.sent));
    let address = device
        .addresses
        .first()
        .and_then(|address| address.split('/').next())
        .unwrap_or("—");
    let gateway = device.gateway.as_deref().unwrap_or("—");
    column![
        pair(skin, "PING", ping, "PERDA DE PACOTES", loss),
        pair(skin, "RECEBENDO", received, "ENVIANDO", sent),
        pair(skin, "BAIXADO", total_received, "ENVIADO", total_sent),
        pair(
            skin,
            "ENDEREÇO IP",
            address.to_owned(),
            "GATEWAY",
            gateway.to_owned()
        ),
    ]
    .spacing(8)
    .into()
}

fn ping(ping: Option<Ping>) -> String {
    match ping {
        Some(Ping::Reply(milliseconds)) => format!("{milliseconds} ms"),
        Some(Ping::Lost) => "sem resposta".to_owned(),
        Some(Ping::Unavailable) => "indisponível".to_owned(),
        None => "—".to_owned(),
    }
}

fn pair<'a>(
    skin: &Skin,
    left_label: &'static str,
    left_value: String,
    right_label: &'static str,
    right_value: String,
) -> Element<'a, Message> {
    row![
        cell(skin, left_label, left_value),
        cell(skin, right_label, right_value)
    ]
    .spacing(12)
    .into()
}

fn cell<'a>(skin: &Skin, label: &'static str, value: String) -> Element<'a, Message> {
    column![
        text(label).size(10).color(skin.dark_foreground),
        text(value).size(12).color(skin.foreground)
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::ping;
    use crate::domain::telemetry::Ping;

    #[test]
    fn ping_distingue_espera_perda_e_indisponibilidade() {
        assert_eq!(ping(None), "—");
        assert_eq!(ping(Some(Ping::Reply(8))), "8 ms");
        assert_eq!(ping(Some(Ping::Lost)), "sem resposta");
        assert_eq!(ping(Some(Ping::Unavailable)), "indisponível");
    }
}
