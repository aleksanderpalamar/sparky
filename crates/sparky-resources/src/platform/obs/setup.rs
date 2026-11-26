use std::path::PathBuf;

use serde_json::Value;

#[derive(Clone, PartialEq, Eq)]
pub struct Server {
    pub port: u16,
    pub password: Option<String>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Server")
            .field("port", &self.port)
            .field("password", &self.password.as_ref().map(|_| "<oculta>"))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setup {
    Disabled,
    Enabled(Server),
}

pub fn read(text: Option<&str>) -> Setup {
    let Some(config) = text.and_then(|text| serde_json::from_str::<Value>(text).ok()) else {
        return Setup::Disabled;
    };
    if config["server_enabled"].as_bool() != Some(true) {
        return Setup::Disabled;
    }
    let Some(port) = config["server_port"]
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
    else {
        return Setup::Disabled;
    };
    let password = (config["auth_required"].as_bool() == Some(true))
        .then(|| config["server_password"].as_str().map(str::to_owned))
        .flatten();
    Setup::Enabled(Server { port, password })
}

pub fn load() -> Setup {
    let text = location().and_then(|path| std::fs::read_to_string(path).ok());
    read(text.as_deref())
}

fn location() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|path| path.starts_with('/'))
        .map(PathBuf::from)
        .or_else(|| {
            let home = std::env::var("HOME")
                .ok()
                .filter(|path| path.starts_with('/'))?;
            Some(PathBuf::from(home).join(".config"))
        })?;
    Some(base.join("obs-studio/plugin_config/obs-websocket/config.json"))
}
