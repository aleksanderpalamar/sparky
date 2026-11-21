use serde::{Deserialize, Serialize};

pub const MIN_SIZE: i32 = 40;
pub const MAX_SIZE: i32 = 96;
const DEFAULT_SIZE: i32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Dock {
    pub enabled: bool,
    pub size: i32,
    pub autohide: bool,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            enabled: true,
            size: DEFAULT_SIZE,
            autohide: false,
        }
    }
}

impl Dock {
    pub fn slot(&self) -> i32 {
        self.size.clamp(MIN_SIZE, MAX_SIZE)
    }
}
