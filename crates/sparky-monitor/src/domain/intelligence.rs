use serde::Serialize;
use sparky_resources::application::Insight;
use sparky_resources::orchestration::Decision;
use sparky_resources::protocol::Status;

#[derive(Debug, Clone, PartialEq)]
pub enum Advice {
    Unavailable,
    Ready(Box<Briefing>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Briefing {
    pub status: Status,
    pub metrics: Vec<Insight>,
    pub decisions: Vec<Decision>,
}

pub fn percent(value: Option<f32>) -> Option<String> {
    value
        .filter(|value| value.is_finite())
        .map(|value| format!("{value:.1}%"))
}

pub fn score(value: Option<f32>) -> Option<String> {
    value
        .filter(|value| value.is_finite())
        .map(|value| format!("{value:.1}"))
}

pub fn clock(seconds: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

pub fn key(prefix: &str, value: &impl Serialize) -> String {
    let name = match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        Ok(serde_json::Value::Object(fields)) => {
            match fields.get(prefix).and_then(|tag| tag.as_str()) {
                Some(tag) => tag.to_owned(),
                None => fields.keys().next().cloned().unwrap_or_default(),
            }
        }
        _ => String::new(),
    };
    format!("{prefix}-{}", name.replace('_', "-"))
}
