use super::tray::MenuEntry;

pub const ADDRESS: &str = "sparky.bluetooth";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Adapter {
    pub path: String,
    pub name: String,
    pub powered: bool,
    pub discovering: bool,
    pub devices: Vec<Device>,
    pub connected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub path: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
    pub kind: DeviceKind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DeviceKind {
    Phone,
    Display,
    Audio,
    Computer,
    #[default]
    Other,
}

impl DeviceKind {
    pub fn from_bluez(icon: Option<&str>) -> Self {
        match icon {
            Some("phone" | "smartphone") => Self::Phone,
            Some("video-display" | "television") => Self::Display,
            Some("audio-headset" | "audio-headphones" | "audio-card" | "audio-speakers") => {
                Self::Audio
            }
            Some("computer" | "input-tablet") => Self::Computer,
            _ => Self::Other,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Phone => "phone-symbolic",
            Self::Display => "video-display-symbolic",
            Self::Audio => "audio-headphones-symbolic",
            Self::Computer => "computer-symbolic",
            Self::Other => "bluetooth-symbolic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothRequest {
    Power { adapter: String, enabled: bool },
    Discover(String),
    StopDiscovery(String),
    Connect(String),
    Disconnect(String),
    Pair { device: String, name: String },
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bluetooth {
    Unavailable,
    Ready(Vec<Adapter>),
}

impl Bluetooth {
    pub fn icon(&self) -> &'static str {
        let Self::Ready(adapters) = self else {
            return "bluetooth-disabled-symbolic";
        };
        if !adapters.iter().any(|adapter| adapter.powered) {
            return "bluetooth-disabled-symbolic";
        }
        if adapters
            .iter()
            .any(|adapter| adapter.powered && !adapter.connected.is_empty())
        {
            return "bluetooth-active-symbolic";
        }
        "bluetooth-symbolic"
    }

    pub fn title(&self) -> &'static str {
        match self.icon() {
            "bluetooth-disabled-symbolic" if matches!(self, Self::Unavailable) => {
                "Bluetooth — serviço indisponível"
            }
            "bluetooth-disabled-symbolic" => "Bluetooth desligado",
            "bluetooth-active-symbolic" => "Bluetooth conectado",
            _ => "Bluetooth ligado",
        }
    }

    pub fn menu(&self) -> Vec<MenuEntry> {
        let mut labels = vec![self.title().to_owned()];
        if let Self::Ready(adapters) = self {
            labels.extend(adapters.iter().flat_map(|adapter| {
                let status = match adapter.powered {
                    true => "ligado",
                    false => "desligado",
                };
                std::iter::once(format!("{} — {status}", adapter.name)).chain(
                    adapter
                        .connected
                        .iter()
                        .map(|name| format!("Conectado: {name}")),
                )
            }));
        }
        labels
            .into_iter()
            .enumerate()
            .map(|(id, label)| MenuEntry {
                id: id as i32,
                label,
                enabled: false,
                separator: false,
            })
            .collect()
    }
}
