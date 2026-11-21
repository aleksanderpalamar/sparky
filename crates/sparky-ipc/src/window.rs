use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub app_id: String,
    pub pid: Option<u32>,
    pub workspace: Option<u32>,
    pub visible: bool,
    pub focused: bool,
    pub fullscreen: bool,
}
