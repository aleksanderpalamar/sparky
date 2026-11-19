use serde::{Deserialize, Serialize};

use crate::application::{Insight, Progress};
use crate::domain::{Activities, Mode, ServiceState, Switch, TrainingState};
use crate::orchestration::Decision;

pub const VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub version: u16,
    #[serde(flatten)]
    pub body: T,
}

impl<T> Envelope<T> {
    pub fn current(body: T) -> Self {
        Self {
            version: VERSION,
            body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    Status,
    Insight,
    Decisions,
    SetMode { mode: Mode },
    SetEnabled { switch: Switch },
    SetLearning { switch: Switch },
    ClearSafeMode,
    ForgetKnowledge,
    SetProtected { names: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    Status(Status),
    Insight { metrics: Vec<Insight> },
    Decisions { entries: Vec<Decision> },
    Done,
    Refused { reason: Refusal },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub service: String,
    pub state: ServiceState,
    pub enabled: Switch,
    pub mode: Mode,
    pub learning: Switch,
    pub training: TrainingState,
    pub activities: Activities,
    #[serde(default)]
    pub progress: Progress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refusal {
    UnsupportedVersion,
    Malformed,
    TooLarge,
    NotInSafeMode,
    InvalidNames,
}
