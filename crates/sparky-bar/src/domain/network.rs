use super::secret::Secret;

pub const ADDRESS: &str = "sparky.network";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Security {
    Open,
    Personal,
    Sae,
    Enterprise,
    Unsupported,
}

impl Security {
    pub fn validate(self, password: &str) -> Result<(), String> {
        let valid = match self {
            Self::Open => true,
            Self::Personal => {
                (8..=63).contains(&password.len())
                    || (password.len() == 64
                        && password.bytes().all(|byte| byte.is_ascii_hexdigit()))
            }
            Self::Sae => (1..=63).contains(&password.len()),
            Self::Enterprise => {
                return Err(
                    "Redes corporativas exigem um perfil previamente configurado".to_owned(),
                );
            }
            Self::Unsupported => {
                return Err("Este tipo de segurança Wi-Fi não é suportado pelo painel".to_owned());
            }
        };
        match valid {
            true => Ok(()),
            false => Err("Senha Wi-Fi inválida para o tipo de segurança da rede".to_owned()),
        }
    }

    pub fn from_flags(flags: u32, wpa: u32, rsn: u32) -> Self {
        match (wpa | rsn, flags & 1) {
            (0, 0) => Self::Open,
            (value, _) if value & 0x100 != 0 => Self::Personal,
            (value, _) if value & 0x400 != 0 => Self::Sae,
            (value, _) if value & (0x200 | 0x2000) != 0 => Self::Enterprise,
            _ => Self::Unsupported,
        }
    }

    pub fn matches_flags(self, flags: u32, wpa: u32, rsn: u32) -> bool {
        let modes = wpa | rsn;
        match self {
            Self::Personal => modes & 0x100 != 0,
            Self::Sae => modes & 0x400 != 0,
            Self::Enterprise => modes & (0x200 | 0x2000) != 0,
            _ => self == Self::from_flags(flags, wpa, rsn),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiNetwork {
    pub path: String,
    pub device: String,
    pub ssid: Vec<u8>,
    pub strength: u8,
    pub security: Security,
    pub profile: Option<String>,
    pub active: bool,
}

impl WifiNetwork {
    pub fn label(&self) -> String {
        String::from_utf8_lossy(&self.ssid).into_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDevice {
    pub path: String,
    pub interface: String,
    pub kind: u32,
    pub state: u32,
    pub addresses: Vec<String>,
    pub gateway: Option<String>,
    pub active_connection: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NetworkState {
    pub enabled: bool,
    pub wireless: bool,
    pub hardware_wireless: bool,
    pub connectivity: u32,
    pub devices: Vec<NetworkDevice>,
    pub networks: Vec<WifiNetwork>,
}

impl NetworkState {
    pub fn icon(&self) -> &'static str {
        let wifi = self.networks.iter().find(|network| network.active);
        if let Some(network) = wifi {
            return match network.strength {
                0..=24 => "network-wireless-signal-weak-symbolic",
                25..=49 => "network-wireless-signal-ok-symbolic",
                50..=74 => "network-wireless-signal-good-symbolic",
                _ => "network-wireless-signal-excellent-symbolic",
            };
        }
        match self.devices.iter().any(|device| device.state == 100) {
            true => "network-wired-symbolic",
            false => "network-offline-symbolic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Network {
    Unavailable(String),
    Ready(NetworkState),
}

impl Default for Network {
    fn default() -> Self {
        Self::Unavailable("Aguardando o NetworkManager".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkRequest {
    Wireless(bool),
    Scan(String),
    Disconnect(String),
    Activate {
        profile: String,
        device: String,
        access_point: String,
    },
    Join {
        network: WifiNetwork,
        password: Secret,
        remember: bool,
    },
}
