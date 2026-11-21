use serde::Serialize;
use sparky_resources::application::{Insight, Round};
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

pub fn fraction(done: impl std::fmt::Display, total: impl std::fmt::Display) -> String {
    format!("{done} / {total}")
}

pub fn errors(round: &Round) -> String {
    let neural = round
        .neural
        .filter(|value| value.is_finite())
        .map_or_else(|| "—".to_owned(), |value| format!("{value:.2}"));
    format!("{neural} / {:.2}", round.statistical)
}

pub fn ceiling(values: impl IntoIterator<Item = f64>) -> f64 {
    let peak = values
        .into_iter()
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let magnitude = 10f64.powf(peak.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .find(|step| peak <= step * magnitude)
        .unwrap_or(10.0);
    step * magnitude
}
