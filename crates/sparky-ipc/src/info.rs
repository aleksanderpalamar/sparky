use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    #[default]
    Unknown,
    Real,
    Nested,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SessionInfo {
    pub version: String,
    pub display: String,
    #[serde(default)]
    pub session: SessionKind,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub accent: String,
    #[serde(default)]
    pub swatches: Vec<String>,
}
