use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Switch {
    #[default]
    On,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Adaptive,
    #[default]
    Observation,
    Conservative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Disabled,
    Starting,
    Observing,
    Adaptive,
    Conservative,
    Degraded,
    Safe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    None,
    Conservative,
    Adaptive,
}

impl ServiceState {
    pub fn authority(self) -> Authority {
        match self {
            Self::Adaptive => Authority::Adaptive,
            Self::Conservative => Authority::Conservative,
            _ => Authority::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "training", content = "reason")]
pub enum TrainingState {
    Allowed,
    Active,
    Suspended(Suspension),
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suspension {
    UserChoice,
    SensitiveActivity,
    Pressure,
    Budget,
}
