use iced::widget::{column, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::{action, network_metrics, parts};
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::network::{Network, NetworkDevice, NetworkRequest, NetworkState, Security};
use crate::domain::panel::PanelKind;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Network::Ready(state) = &panels.network else {
        return text("NetworkManager indisponível")
            .size(12)
            .color(skin.foreground)
            .into();
    };
    if let Some(wifi) = &panels.wifi {
        return super::forms::wifi(skin, wifi);
    }
    let active = state.devices.iter().find(|device| device.state == 100);
    let mut body = column![heading(skin, state, active)].spacing(10);
    if let Some(device) = active {
        body = body.push(network_metrics::view(skin, device, panels.metrics.as_ref()));
    }
    body = body.push(action(
        skin,
        "Configurações de rede",
        Message::OpenNetworkSettings,
    ));
    for device in &state.devices {
        body = body.push(device_actions(skin, device));
    }
    body.push(wireless(skin, state, panels)).into()
}

fn heading<'a>(
    skin: &Skin,
    state: &NetworkState,
    active: Option<&NetworkDevice>,
) -> Element<'a, Message> {
    let Some(device) = active else {
        return parts::heading(
            skin,
            "network-offline-symbolic",
            "Rede".to_owned(),
            if state.enabled {
                "Desconectado"
            } else {
                "Rede desativada"
            }
            .to_owned(),
            None,
        );
    };
    let icon = if device.kind == 1 {
        "network-wired-symbolic"
    } else {
        state.icon()
    };
    let speed = device
        .speed
        .map_or(String::new(), |speed| format!(" · {speed} Mb/s"));
    parts::heading(
        skin,
        icon,
        device
            .name
            .clone()
            .unwrap_or_else(|| device.interface.clone()),
        format!(
            "{} · {}{speed}",
            if device.kind == 1 { "Cabo" } else { "Wi-Fi" },
            device.interface
        ),
        None,
    )
}

fn device_actions<'a>(skin: &Skin, device: &NetworkDevice) -> Element<'a, Message> {
    let mut body = column![].spacing(4);
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
    body.into()
}

fn wireless<'a>(skin: &Skin, state: &'a NetworkState, panels: &Panels) -> Element<'a, Message> {
    let devices: Vec<_> = state
        .devices
        .iter()
        .filter(|device| device.kind == 2)
        .collect();
    if devices.is_empty() {
        return text("Nenhum adaptador Wi-Fi disponível")
            .size(11)
            .color(skin.dark_foreground)
            .into();
    }
    let label = if state.wireless {
        "Desligar Wi-Fi"
    } else {
        "Ligar Wi-Fi"
    };
    let mut body = column![action(
        skin,
        label,
        Message::NetworkRequest(NetworkRequest::Wireless(!state.wireless))
    )]
    .spacing(6);
    if !state.hardware_wireless {
        return body
            .push(
                text("Wi-Fi bloqueado pelo hardware")
                    .size(11)
                    .color(skin.urgent),
            )
            .into();
    }
    if !state.wireless {
        return body.into();
    }
    let busy = panels.pending.contains_key(&PanelKind::Network);
    if !busy {
        for device in devices {
            body = body.push(action(
                skin,
                format!("Buscar redes · {}", device.interface),
                Message::NetworkRequest(NetworkRequest::Scan(device.path.clone())),
            ));
        }
    }
    body.push(networks(skin, state, busy)).into()
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
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    row![body.width(Length::Fill)].into()
}
