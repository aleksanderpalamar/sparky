use super::{action, forms};
use crate::{
    app::{message::Message, panel_state::Panels},
    domain::{
        network::{Network, NetworkRequest, NetworkState, Security},
        panel::PanelKind,
    },
};
use iced::{
    Element, Length,
    widget::{column, row, text},
};
use sparky_theme::Skin;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let state = match &panels.network {
        Network::Ready(state) => state,
        Network::Unavailable(error) => return text(error).size(12).color(skin.foreground).into(),
    };
    if let Some(wifi) = &panels.wifi {
        return forms::wifi(skin, wifi);
    }
    let status = match state.connectivity {
        4 => "Conectado à internet",
        2 => "Autenticação na rede necessária",
        3 => "Conexão limitada",
        _ => "Sem acesso à internet",
    };
    let mut body = column![text(status).size(13).color(skin.accent)].spacing(10);
    if !state.enabled {
        body = body.push(
            text("Rede desativada no sistema")
                .color(skin.urgent)
                .size(12),
        );
    }
    for device in &state.devices {
        let state_label = match device.state {
            100 => "conectado",
            40..=90 => "conectando",
            10 | 20 => "indisponível",
            _ => "desconectado",
        };
        body = body.push(
            text(format!("{} · {state_label}", device.interface))
                .size(12)
                .color(skin.foreground),
        );
        for address in &device.addresses {
            body = body.push(text(address).size(12).color(skin.foreground));
        }
        if let Some(gateway) = &device.gateway {
            body = body.push(
                text(format!("Gateway: {gateway}"))
                    .size(11)
                    .color(skin.foreground),
            );
        }
        if device.kind == 1 && device.state == 30 {
            body = body.push(action(
                skin,
                "Conectar cabo de rede",
                Message::NetworkRequest(NetworkRequest::Activate {
                    profile: "/".to_owned(),
                    device: device.path.clone(),
                    access_point: "/".to_owned(),
                }),
            ));
        }
        if device.state == 100 {
            body = body.push(action(
                skin,
                "Desconectar",
                Message::NetworkRequest(NetworkRequest::Disconnect(device.path.clone())),
            ));
        }
    }
    let devices: Vec<_> = state
        .devices
        .iter()
        .filter(|device| device.kind == 2)
        .collect();
    if devices.is_empty() {
        return body
            .push(
                text("Nenhum adaptador Wi-Fi disponível")
                    .size(12)
                    .color(skin.dark_foreground),
            )
            .into();
    }
    let label = if state.wireless {
        "Desligar Wi-Fi"
    } else {
        "Ligar Wi-Fi"
    };
    body = body.push(action(
        skin,
        label,
        Message::NetworkRequest(NetworkRequest::Wireless(!state.wireless)),
    ));
    if !state.hardware_wireless {
        return body
            .push(
                text("Wi-Fi bloqueado pelo hardware")
                    .size(12)
                    .color(skin.urgent),
            )
            .into();
    }
    if !state.wireless {
        return body.into();
    }
    if !panels.pending.contains_key(&PanelKind::Network) {
        for device in devices {
            body = body.push(action(
                skin,
                format!("Buscar redes · {}", device.interface),
                Message::NetworkRequest(NetworkRequest::Scan(device.path.clone())),
            ));
        }
    }
    body.push(networks(
        skin,
        state,
        panels.pending.contains_key(&PanelKind::Network),
    ))
    .into()
}

fn networks<'a>(skin: &Skin, state: &'a NetworkState, busy: bool) -> Element<'a, Message> {
    let mut body = column![].spacing(6);
    for network in &state.networks {
        let secure = match network.security {
            Security::Open => "aberta",
            Security::Personal => "WPA2",
            Security::Sae => "WPA3",
            Security::Enterprise => "corporativa",
            Security::Unsupported => "não suportada",
        };
        let label = format!(
            "{} {} · {}% · {secure}",
            if network.active { "●" } else { "○" },
            network.label(),
            network.strength
        );
        if busy {
            body = body.push(text(label).size(12).color(skin.dark_foreground));
            continue;
        }
        body = body.push(action(skin, label, Message::ChooseWifi(network.clone())));
    }
    if state.networks.is_empty() {
        body = body.push(
            text("Nenhuma rede encontrada; use Buscar redes")
                .size(12)
                .color(skin.dark_foreground),
        );
    }
    row![body.width(Length::Fill)].into()
}
