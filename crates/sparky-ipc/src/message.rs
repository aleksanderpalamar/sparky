use serde::{Deserialize, Serialize};

use crate::info::SessionInfo;
use crate::output::OutputInfo;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    Workspaces,
    FocusWorkspace { id: u32 },
    Subscribe,
    ReloadConfig,
    Capture { area: CaptureArea, path: String },
    Outputs,
    Session { action: SessionAction },
    Idle { hold: IdleHold },
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdleHold {
    #[default]
    Free,
    Held,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Lock,
    Unlock,
    Suspend,
    Logout,
    Reboot,
    Poweroff,
}

impl SessionAction {
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "lock" => Some(SessionAction::Lock),
            "unlock" => Some(SessionAction::Unlock),
            "suspend" => Some(SessionAction::Suspend),
            "logout" => Some(SessionAction::Logout),
            "reboot" => Some(SessionAction::Reboot),
            "poweroff" => Some(SessionAction::Poweroff),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureArea {
    Output {
        output: Option<String>,
    },
    Window,
    Region {
        output: Option<String>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    Done,
    Workspaces { workspaces: Vec<WorkspaceInfo> },
    Outputs { outputs: Vec<OutputInfo> },
    Info { info: SessionInfo },
    Event { event: Event },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    WorkspacesChanged,
    Locked,
    Unlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: u32,
    pub monitor: String,
    pub windows: usize,
    pub visible: bool,
    pub focused: bool,
}
