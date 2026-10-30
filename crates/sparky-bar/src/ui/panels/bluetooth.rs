use iced::Element;
use iced::widget::{column, text};
use sparky_theme::Skin;

use super::{action, parts};
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::bluetooth::{Adapter, Bluetooth, BluetoothRequest, Device};
use crate::domain::panel::PanelKind;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Some(Bluetooth::Ready(adapters)) = &panels.bluetooth else {
        return column![
            parts::heading(
                skin,
                "bluetooth-symbolic",
                "Bluetooth".to_owned(),
                "Serviço indisponível".to_owned(),
                None,
            ),
            text("Nenhum adaptador Bluetooth disponível").size(12),
        ]
        .spacing(12)
        .into();
    };
    let busy = panels.pending.contains_key(&PanelKind::Bluetooth);
    let mut body = column![].spacing(12);
    for adapter in adapters {
        body = body.push(adapter_view(skin, adapter, busy));
    }
    body.into()
}

fn adapter_view<'a>(skin: &Skin, adapter: &'a Adapter, busy: bool) -> Element<'a, Message> {
    let status = match (adapter.powered, adapter.discovering) {
        (false, _) => "Desligado",
        (true, true) => "Procurando aparelhos...",
        (true, false) => "Ligado",
    };
    let toggle = Message::BluetoothRequest(BluetoothRequest::Power {
        adapter: adapter.path.clone(),
        enabled: !adapter.powered,
    });
    let mut body = column![parts::heading(
        skin,
        "bluetooth-symbolic",
        "Bluetooth".to_owned(),
        status.to_owned(),
        Some((adapter.powered, toggle)),
    )]
    .spacing(10);
    if !adapter.powered {
        return body.into();
    }
    body = body.push(parts::section(skin, "PAREADOS"));
    let paired: Vec<_> = adapter
        .devices
        .iter()
        .filter(|device| device.paired)
        .collect();
    body = device_list(body, skin, &paired, busy);
    if paired.is_empty() {
        body = body.push(
            text("Nenhum aparelho pareado")
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    body = body.push(parts::section(skin, "DISPONÍVEIS"));
    let available: Vec<_> = adapter
        .devices
        .iter()
        .filter(|device| !device.paired)
        .collect();
    body = device_list(body, skin, &available, busy);
    if available.is_empty() {
        body = body.push(
            text("Nenhum aparelho encontrado")
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    if !adapter.discovering && !busy {
        body = body.push(action(
            skin,
            "Buscar aparelhos",
            Message::BluetoothRequest(BluetoothRequest::Discover(adapter.path.clone())),
        ));
    }
    body.into()
}

fn device_list<'a>(
    mut body: iced::widget::Column<'a, Message>,
    skin: &Skin,
    devices: &[&'a Device],
    busy: bool,
) -> iced::widget::Column<'a, Message> {
    for device in devices {
        let (trailing, request) = match (device.connected, device.paired) {
            (true, _) => (
                Some("desconectar"),
                BluetoothRequest::Disconnect(device.path.clone()),
            ),
            (_, true) => (
                Some("conectar"),
                BluetoothRequest::Connect(device.path.clone()),
            ),
            _ => (
                Some("parear"),
                BluetoothRequest::Pair {
                    device: device.path.clone(),
                    name: device.name.clone(),
                },
            ),
        };
        body = body.push(parts::choice(
            skin,
            device.kind.icon(),
            device.name.clone(),
            trailing,
            device.connected,
            (!busy).then_some(Message::BluetoothRequest(request)),
        ));
    }
    body
}
