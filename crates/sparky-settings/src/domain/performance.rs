use sparky_config::ResourceMode;
use sparky_resources::domain::Mode;
use sparky_resources::protocol::Status;

use crate::ports::resources::ResourceService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Knob {
    Enabled,
    Learning,
    Notify,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Performance {
    pub status: Option<Status>,
    pub error: Option<String>,
    pub confirming: bool,
    pub advanced: bool,
}

impl Performance {
    pub fn load(service: &dyn ResourceService) -> Self {
        match service.status() {
            Ok(status) => Self {
                status: Some(status),
                error: None,
                ..Self::default()
            },
            Err(error) => Self {
                status: None,
                error: Some(error),
                ..Self::default()
            },
        }
    }
}

pub const MODES: [ResourceMode; 3] = [
    ResourceMode::Adaptive,
    ResourceMode::Observation,
    ResourceMode::Conservative,
];

pub fn mode(chosen: ResourceMode) -> Mode {
    match chosen {
        ResourceMode::Adaptive => Mode::Adaptive,
        ResourceMode::Observation => Mode::Observation,
        ResourceMode::Conservative => Mode::Conservative,
    }
}

pub fn mode_key(chosen: ResourceMode) -> &'static str {
    match chosen {
        ResourceMode::Adaptive => "adaptive",
        ResourceMode::Observation => "observation",
        ResourceMode::Conservative => "conservative",
    }
}

pub fn tag(value: &impl serde::Serialize, field: &str) -> String {
    let name = match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        Ok(serde_json::Value::Object(fields)) => fields
            .get(field)
            .and_then(|tag| tag.as_str())
            .unwrap_or_default()
            .to_owned(),
        _ => String::new(),
    };
    name.replace('_', "-")
}
