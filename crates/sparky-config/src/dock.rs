use serde::{Deserialize, Serialize};

pub const MIN_SIZE: i32 = 40;
pub const MAX_SIZE: i32 = 96;
pub const MAX_RADIUS: i32 = 16;
const DEFAULT_SIZE: i32 = 64;
const DEFAULT_RADIUS: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Dock {
    pub enabled: bool,
    pub size: i32,
    pub autohide: bool,
    pub rounded: bool,
    pub radius: i32,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            enabled: true,
            size: DEFAULT_SIZE,
            autohide: false,
            rounded: false,
            radius: DEFAULT_RADIUS,
        }
    }
}

impl Dock {
    pub fn slot(&self) -> i32 {
        self.size.clamp(MIN_SIZE, MAX_SIZE)
    }

    pub fn corner(&self) -> i32 {
        match self.rounded {
            true => self.radius.clamp(0, MAX_RADIUS),
            false => 0,
        }
    }
}
