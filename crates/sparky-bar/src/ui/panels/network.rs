use fluent_bundle::FluentArgs;
use iced::widget::{column, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;

use super::{action, network_metrics, parts};
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::network::{
    DeviceKind, DeviceState, Network, NetworkDevice, NetworkRequest, NetworkState, Security,
};
use crate::domain::panel::PanelKind;
use crate::i18n;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Network::Ready(state) = &panels.network else {
        return text(i18n::text("network-manager-unavailable"))
            .size(12)
            .color(skin.foreground)
            .into();
    };
    if let Some(wifi) = &panels.wifi {
        return super::forms::wifi(skin, wifi);
    }
    let active = state
        .devices
        .iter()
        .find(|device| device.state == DeviceState::Activated);
    let mut body = column![heading(skin, state, active)].spacing(10);
    if let Some(device) = active {
        body = body.push(network_metrics::view(skin, device, panels.metrics.as_ref()));
    }
    body = body.push(action(
        skin,
        i18n::text("network-settings"),
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
            i18n::text("network"),
            i18n::text(if state.enabled {
                "disconnected"
            } else {
                "network-disabled"
            }),
            None,
        );
    };
    let (icon, medium) = match device.kind {
        DeviceKind::Ethernet => ("network-wired-symbolic", i18n::text("wired")),
        DeviceKind::Wifi => (state.icon(), i18n::text("wifi")),
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
        format!("{medium} · {}{speed}", device.interface),
        None,
    )
}

fn device_actions<'a>(skin: &Skin, device: &NetworkDevice) -> Element<'a, Message> {
    let mut body = column![].spacing(4);
    if device.kind == DeviceKind::Ethernet && device.state == DeviceState::Disconnected {
        body = body.push(action(
            skin,
            i18n::text("connect-wired"),
            Message::NetworkRequest(NetworkRequest::Activate {
                profile: "/".to_owned(),
                device: device.path.clone(),
                access_point: "/".to_owned(),
            }),
        ));
    }
    if device.state == DeviceState::Activated {
        body = body.push(action(
            skin,
            i18n::text("disconnect"),
            Message::NetworkRequest(NetworkRequest::Disconnect(device.path.clone())),
        ));
    }
    body.into()
}

fn wireless<'a>(skin: &Skin, state: &'a NetworkState, panels: &Panels) -> Element<'a, Message> {
    let devices: Vec<_> = state
        .devices
        .iter()
        .filter(|device| device.kind == DeviceKind::Wifi)
        .collect();
    if devices.is_empty() {
        return text(i18n::text("no-wifi-adapter"))
            .size(11)
            .color(skin.dark_foreground)
            .into();
    }
    let label = if state.wireless {
        "turn-off-wifi"
    } else {
        "turn-on-wifi"
    };
    let mut body = column![action(
        skin,
        i18n::text(label),
        Message::NetworkRequest(NetworkRequest::Wireless(!state.wireless))
    )]
    .spacing(6);
    if !state.hardware_wireless {
        return body
            .push(
                text(i18n::text("wifi-hardware-blocked"))
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
                scan_label(&device.interface),
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
            Security::Open => "open-network",
            Security::Personal => "WPA2",
            Security::Sae => "WPA3",
            Security::Enterprise => "enterprise-network",
            Security::Unsupported => "unsupported-network",
        };
        let label = format!(
            "{} {} · {}% · {secure}",
            if network.active { "●" } else { "○" },
            network.label(),
            network.strength,
            secure = i18n::text(secure)
        );
        if busy {
            body = body.push(text(label).size(12).color(skin.dark_foreground));
            continue;
        }
        body = body.push(action(skin, label, Message::ChooseWifi(network.clone())));
    }
    if state.networks.is_empty() {
        body = body.push(
            text(i18n::text("no-networks"))
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    row![body.width(Length::Fill)].into()
}

fn scan_label(interface: &str) -> String {
    let mut args = FluentArgs::new();
    args.set("interface", interface.to_owned());
    i18n::message("scan-networks", Some(&args))
}
