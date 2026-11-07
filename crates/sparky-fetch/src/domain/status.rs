use serde::{Serialize, Serializer};
use sparky_theme::Rgb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Text(String),
    Lines(Vec<String>),
    Colors(Vec<Rgb>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Missing,
    NoCompositor,
    Unsupported,
    Uncollected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Status {
    Ready { value: Value },
    Unavailable { reason: Reason },
    TimedOut,
    Failed { error: String },
}

impl Status {
    pub fn ready(value: Value) -> Self {
        Status::Ready { value }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self::ready(Value::Text(text.into()))
    }

    pub fn lines(lines: Vec<String>) -> Self {
        Self::ready(Value::Lines(lines))
    }

    pub fn unavailable(reason: Reason) -> Self {
        Status::Unavailable { reason }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Status::Failed {
            error: error.into(),
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match self {
            Status::Ready { value } => Some(value),
            Status::Unavailable { .. } | Status::TimedOut | Status::Failed { .. } => None,
        }
    }
}

pub fn hex(color: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Text(text) => serializer.serialize_str(text),
            Value::Lines(lines) => lines.serialize(serializer),
            Value::Colors(colors) => colors
                .iter()
                .map(|color| hex(*color))
                .collect::<Vec<_>>()
                .serialize(serializer),
        }
    }
}
