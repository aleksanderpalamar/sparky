use super::action;
use crate::{
    app::{message::Message, panel_state::Panels},
    domain::{
        bluetooth::{Bluetooth, BluetoothRequest},
        panel::PanelKind,
    },
};
use iced::{
    Element,
    widget::{column, text},
};
use sparky_theme::Skin;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Some(Bluetooth::Ready(adapters)) = &panels.bluetooth else {
        return text("Nenhum adaptador Bluetooth disponível")
            .size(12)
            .color(skin.foreground)
            .into();
    };
    let mut body = column![].spacing(10);
    let busy = panels.pending.contains_key(&PanelKind::Bluetooth);
    for adapter in adapters {
        body = body.push(text(&adapter.name).size(13).color(skin.accent));
        let label = if adapter.powered {
            "Desligar Bluetooth"
        } else {
            "Ligar Bluetooth"
        };
        if !busy {
            body = body.push(action(
                skin,
                label,
                Message::BluetoothRequest(BluetoothRequest::Power {
                    adapter: adapter.path.clone(),
                    enabled: !adapter.powered,
                }),
            ));
        }
        if !adapter.powered {
            body = body.push(text("Desligado").size(12).color(skin.dark_foreground));
            continue;
        }
        if adapter.discovering {
            body = body.push(
                text("Busca de dispositivos ativa no sistema")
                    .size(11)
                    .color(skin.dark_foreground),
            );
        }
        if !busy {
            body = body.push(action(
                skin,
                "Buscar dispositivos por 15 segundos",
                Message::BluetoothRequest(BluetoothRequest::Discover(adapter.path.clone())),
            ));
        }
        for device in &adapter.devices {
            let status = match (device.connected, device.paired) {
                (true, _) => "conectado",
                (_, true) => "pareado",
                _ => "novo",
            };
            let label = format!("{} · {status}", device.name);
            if busy {
                body = body.push(text(label).size(12).color(skin.dark_foreground));
                continue;
            }
            let request = match (device.connected, device.paired) {
                (true, _) => BluetoothRequest::Disconnect(device.path.clone()),
                (_, true) => BluetoothRequest::Connect(device.path.clone()),
                _ => BluetoothRequest::Pair {
                    device: device.path.clone(),
                    name: device.name.clone(),
                },
            };
            body = body.push(action(skin, label, Message::BluetoothRequest(request)));
        }
        if adapter.devices.is_empty() {
            body = body.push(
                text("Use Buscar dispositivos para encontrar novos aparelhos")
                    .size(12)
                    .color(skin.dark_foreground),
            );
        }
    }
    body.into()
}
