use super::panel::PanelKind;
use super::secret::Secret;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PromptId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptKind {
    Wifi {
        ssid: String,
    },
    Confirm {
        device: String,
        code: Option<u32>,
    },
    Pin {
        device: String,
        numeric: bool,
    },
    Display {
        device: String,
        code: String,
        entered: u16,
    },
}

impl PromptKind {
    pub fn panel(&self) -> PanelKind {
        match self {
            Self::Wifi { .. } => PanelKind::Network,
            _ => PanelKind::Bluetooth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub id: PromptId,
    pub kind: PromptKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    Accept,
    Reject,
    Secret(Secret),
}

#[derive(Debug, Clone)]
pub enum PromptEvent {
    Show(Prompt),
    Dismiss(PromptId),
}
