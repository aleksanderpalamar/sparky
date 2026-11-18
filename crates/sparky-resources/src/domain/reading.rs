use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Reading<T> {
    Missing,
    Measured { value: T, age_ms: u64 },
    Estimated { value: T },
    Stale { value: T, age_ms: u64 },
}

impl<T> Reading<T> {
    pub fn observed(value: T, age_ms: u64, freshness_ms: u64) -> Self {
        if age_ms > freshness_ms {
            return Self::Stale { value, age_ms };
        }
        Self::Measured { value, age_ms }
    }

    pub fn measured(&self) -> Option<&T> {
        match self {
            Self::Measured { value, .. } => Some(value),
            _ => None,
        }
    }
}
