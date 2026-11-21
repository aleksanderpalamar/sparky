use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identity {
    pub pid: u32,
    pub started: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoClass {
    None,
    BestEffort(u8),
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "change", content = "value")]
pub enum Change {
    Nice(i8),
    IoPriority(IoClass),
    Affinity(Vec<u32>),
}

impl Change {
    pub fn kind(&self) -> ChangeKind {
        match self {
            Self::Nice(_) => ChangeKind::Nice,
            Self::IoPriority(_) => ChangeKind::IoPriority,
            Self::Affinity(_) => ChangeKind::Affinity,
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Nice(value) => format!("nice {value}"),
            Self::Affinity(cpus) => {
                let list: Vec<String> = cpus.iter().map(u32::to_string).collect();
                format!("CPU {}", list.join(", "))
            }
            Self::IoPriority(IoClass::None) => "io none".to_owned(),
            Self::IoPriority(IoClass::BestEffort(level)) => format!("io best-effort {level}"),
            Self::IoPriority(IoClass::Idle) => "io idle".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Nice,
    IoPriority,
    Affinity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adjustment {
    pub target: Identity,
    pub change: Change,
}
