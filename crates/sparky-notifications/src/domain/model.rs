use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub key: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct Incoming {
    pub sender: String,
    pub replaces: u32,
    pub app: String,
    pub icon: String,
    pub picture: Option<Picture>,
    pub title: String,
    pub body: String,
    pub actions: Vec<Action>,
    pub urgency: Urgency,
    pub timeout: i32,
    pub transient: bool,
    pub resident: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub app: String,
    pub icon: String,
    pub picture: Option<Picture>,
    pub title: String,
    pub body: String,
    pub actions: Vec<Action>,
    pub urgency: Urgency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,
    pub created_ms: u64,
    pub notification: Notification,
    pub active: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub revision: u64,
    pub dnd: bool,
    pub items: Vec<Card>,
}

#[derive(Debug, Clone, Copy)]
pub struct Moment {
    pub elapsed_ms: u64,
    pub utc_ms: u64,
}

impl From<u64> for Moment {
    fn from(value: u64) -> Self {
        Self {
            elapsed_ms: value,
            utc_ms: value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Full,
    Unknown,
    Sender,
    Poisoned,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "Limite de notificações ativas atingido",
            Self::Unknown => "Notificação indisponível",
            Self::Sender => "Notificação pertence a outro aplicativo",
            Self::Poisoned => "Estado das notificações indisponível",
        })
    }
}

pub(super) struct Active {
    pub sender: String,
    pub card: Card,
    pub deadline: Option<u64>,
    pub popup: bool,
    pub transient: bool,
    pub resident: bool,
}
