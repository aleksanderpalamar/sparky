use iced::Element;
use iced::widget::{column, text};
use sparky_theme::Skin;

use super::{action, bluetooth_rows::Rows, parts};
use crate::app::{message::Message, panel_state::Panels};
use crate::domain::bluetooth::{Adapter, Bluetooth, BluetoothRequest};
use crate::domain::panel::PanelKind;
use crate::i18n;

pub fn view<'a>(skin: &Skin, panels: &'a Panels) -> Element<'a, Message> {
    let Some(Bluetooth::Ready(adapters)) = &panels.bluetooth else {
        return column![
            parts::heading(
                skin,
                "bluetooth-symbolic",
                i18n::text("bluetooth"),
                i18n::text("service-unavailable"),
                None,
            ),
            text(i18n::text("no-bluetooth-adapter")).size(12),
        ]
        .spacing(12)
        .into();
    };
    let busy = panels.pending.contains_key(&PanelKind::Bluetooth);
    let mut body = column![].spacing(12);
    for adapter in adapters {
        let rows = Rows {
            skin: *skin,
            adapter: &adapter.path,
            busy,
            unpairing: panels.unpairing.as_deref(),
        };
        body = body.push(adapter_view(skin, adapter, &rows));
    }
    body.into()
}

fn adapter_view<'a>(skin: &Skin, adapter: &'a Adapter, rows: &Rows<'_>) -> Element<'a, Message> {
    let status = match (adapter.powered, adapter.discovering) {
        (false, _) => "off",
        (true, true) => "searching-devices",
        (true, false) => "on",
    };
    let toggle = Message::BluetoothRequest(BluetoothRequest::Power {
        adapter: adapter.path.clone(),
        enabled: !adapter.powered,
    });
    let mut body = column![parts::heading(
        skin,
        "bluetooth-symbolic",
        i18n::text("bluetooth"),
        i18n::text(status),
        Some((adapter.powered, toggle)),
    )]
    .spacing(10);
    if !adapter.powered {
        return body.into();
    }
    body = body.push(parts::section(skin, i18n::text("paired")));
    let paired: Vec<_> = adapter
        .devices
        .iter()
        .filter(|device| device.paired)
        .collect();
    body = rows.push(body, &paired);
    if paired.is_empty() {
        body = body.push(
            text(i18n::text("no-paired-device"))
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    body = body.push(parts::section(skin, i18n::text("available")));
    let available: Vec<_> = adapter
        .devices
        .iter()
        .filter(|device| !device.paired)
        .collect();
    body = rows.push(body, &available);
    if available.is_empty() {
        body = body.push(
            text(i18n::text("no-device-found"))
                .size(11)
                .color(skin.dark_foreground),
        );
    }
    if !adapter.discovering && !rows.busy {
        body = body.push(action(
            skin,
            i18n::text("search-devices"),
            Message::BluetoothRequest(BluetoothRequest::Discover(adapter.path.clone())),
        ));
    }
    body.into()
}
