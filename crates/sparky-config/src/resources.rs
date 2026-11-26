use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceMode {
    Adaptive,
    #[default]
    Observation,
    Conservative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Resources {
    pub enabled: bool,
    pub mode: ResourceMode,
    pub learning: bool,
    pub notify: bool,
    pub protected: Vec<String>,
    pub obs_guide: bool,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ResourceMode::Observation,
            learning: true,
            notify: false,
            protected: Vec::new(),
            obs_guide: true,
        }
    }
}
