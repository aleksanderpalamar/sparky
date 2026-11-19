use crate::domain::activity::ACTIVITIES;
use crate::domain::{Activities, Features, METRICS};

pub const WINDOW: usize = 16;
pub const HORIZONS: [f64; 2] = [10.0, 60.0];
pub const STEP_INPUTS: usize = METRICS.len() * 2 + 1 + ACTIVITIES.len();
pub const OUTPUTS: usize = METRICS.len() * HORIZONS.len();
const SCALE: f32 = 100.0;
const TIME_SCALE: f64 = 10.0;

pub type Example = (Vec<Vec<f32>>, Vec<Option<f32>>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub at: f64,
    pub features: Features,
    pub activities: Activities,
}

pub fn step_input(sample: &Sample, elapsed: f64) -> Vec<f32> {
    let values = METRICS.iter().map(|metric| {
        sample
            .features
            .get(*metric)
            .map_or(0.0, |value| value / SCALE)
    });
    let masks = METRICS
        .iter()
        .map(|metric| flag(sample.features.get(*metric).is_some()));
    let context = ACTIVITIES
        .iter()
        .map(|activity| flag(sample.activities.contains(*activity)));
    values
        .chain(masks)
        .chain(std::iter::once((elapsed / TIME_SCALE).min(10.0) as f32))
        .chain(context)
        .collect()
}

pub fn window(rows: &[Sample]) -> Vec<Vec<f32>> {
    let mut previous = rows.first().map_or(0.0, |sample| sample.at);
    rows.iter()
        .map(|sample| {
            let input = step_input(sample, sample.at - previous);
            previous = sample.at;
            input
        })
        .collect()
}

pub fn decode(outputs: &[f32], current: &Features) -> Vec<f32> {
    outputs
        .iter()
        .zip(METRICS.iter().cycle())
        .map(|(change, metric)| {
            let base = current.get(*metric).unwrap_or(0.0);
            (base + change * SCALE).clamp(0.0, 100.0)
        })
        .collect()
}

pub fn absolute(targets: &[Option<f32>], current: &Features) -> Vec<Option<f32>> {
    targets
        .iter()
        .zip(METRICS.iter().cycle())
        .map(|(change, metric)| Some(current.get(*metric)? + (*change)? * SCALE))
        .collect()
}

pub fn examples(rows: &[Sample]) -> Vec<Example> {
    (WINDOW..rows.len())
        .filter_map(|end| example_at(rows, end))
        .collect()
}

pub fn example_at(rows: &[Sample], end: usize) -> Option<Example> {
    let start = end.checked_sub(WINDOW)?;
    let now = *rows.get(end - 1)?;
    let future = rows.get(end..)?;
    let targets: Vec<Option<f32>> = HORIZONS
        .iter()
        .flat_map(|horizon| target(future, &now, now.at + horizon, horizon * 0.5))
        .collect();
    targets
        .iter()
        .any(Option::is_some)
        .then(|| (window(&rows[start..end]), targets))
}

fn target(future: &[Sample], now: &Sample, when: f64, tolerance: f64) -> Vec<Option<f32>> {
    let found = future
        .iter()
        .find(|sample| sample.at >= when)
        .filter(|sample| sample.at <= when + tolerance);
    METRICS
        .iter()
        .map(|metric| {
            let later = found?.features.get(*metric)?;
            let current = now.features.get(*metric)?;
            Some((later - current) / SCALE)
        })
        .collect()
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}
