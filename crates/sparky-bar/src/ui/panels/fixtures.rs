use crate::{
    app::{panel_flow_tests::wifi, panel_state::Panels},
    domain::{
        audio::{Audio, AudioApp, AudioDevice, AudioState},
        bluetooth::{Adapter, Bluetooth, Device},
        network::{DeviceKind, DeviceState, Network, NetworkDevice, NetworkState},
    },
};

pub fn ready() -> Panels {
    let mut panels = Panels::default();
    panels.audio = Audio::Ready(AudioState {
        outputs: vec![AudioDevice {
            name: "fone".to_owned(),
            label: "Fone de ouvido USB".to_owned(),
            channels: 2,
            volume: 65,
            muted: false,
        }],
        inputs: vec![AudioDevice {
            name: "mic".to_owned(),
            label: "Microfone USB".to_owned(),
            channels: 1,
            volume: 80,
            muted: false,
        }],
        default_output: Some("fone".to_owned()),
        default_input: Some("mic".to_owned()),
        applications: vec![AudioApp {
            index: 3,
            label: "Jogo".to_owned(),
            channels: 2,
            volume: 100,
            writable: true,
        }],
    });
    panels.network = Network::Ready(NetworkState {
        enabled: true,
        wireless: true,
        hardware_wireless: true,
        connectivity: 4,
        devices: vec![NetworkDevice {
            path: "/wifi/1".to_owned(),
            interface: "wlan0".to_owned(),
            name: Some("Rede de teste".to_owned()),
            speed: None,
            kind: DeviceKind::Wifi,
            state: DeviceState::Activated,
            addresses: vec!["192.168.1.10/24".to_owned()],
            gateway: Some("192.168.1.1".to_owned()),
            active_connection: Some("/connection/1".to_owned()),
        }],
        networks: vec![wifi()],
    });
    panels.bluetooth = Some(Bluetooth::Ready(vec![Adapter {
        path: "/org/bluez/hci0".to_owned(),
        name: "Bluetooth integrado".to_owned(),
        powered: true,
        discovering: false,
        devices: vec![Device {
            path: "/org/bluez/hci0/dev_1".to_owned(),
            name: "Fone Bluetooth".to_owned(),
            paired: true,
            connected: true,
            kind: crate::domain::bluetooth::DeviceKind::Audio,
        }],
        connected: vec!["Fone Bluetooth".to_owned()],
    }]));
    panels
}
