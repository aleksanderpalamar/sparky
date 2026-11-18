use crate::domain::{Features, METRICS};

pub const WINDOW: usize = 16;
pub const HORIZONS: [f64; 2] = [10.0, 60.0];
pub const STEP_INPUTS: usize = METRICS.len() * 2 + 1;
pub const OUTPUTS: usize = METRICS.len() * HORIZONS.len();
const SCALE: f32 = 100.0;
const TIME_SCALE: f64 = 10.0;

pub type Example = (Vec<Vec<f32>>, Vec<Option<f32>>);

pub fn step_input(features: &Features, elapsed: f64) -> Vec<f32> {
    let values = METRICS
        .iter()
        .map(|metric| features.get(*metric).map_or(0.0, |value| value / SCALE));
    let masks = METRICS.iter().map(|metric| {
        if features.get(*metric).is_some() {
            1.0
        } else {
            0.0
        }
    });
    values
        .chain(masks)
        .chain(std::iter::once((elapsed / TIME_SCALE).min(10.0) as f32))
        .collect()
}

pub fn window(rows: &[(f64, Features)]) -> Vec<Vec<f32>> {
    let mut previous = rows.first().map_or(0.0, |(at, _)| *at);
    rows.iter()
        .map(|(at, features)| {
            let input = step_input(features, at - previous);
            previous = *at;
            input
        })
        .collect()
}

pub fn decode(outputs: &[f32]) -> Vec<f32> {
    outputs
        .iter()
        .map(|value| (value * SCALE).clamp(0.0, 100.0))
        .collect()
}

pub fn examples(rows: &[(f64, Features)]) -> Vec<Example> {
    (WINDOW..rows.len())
        .filter_map(|end| example_at(rows, end))
        .collect()
}

pub fn example_at(rows: &[(f64, Features)], end: usize) -> Option<Example> {
    let start = end.checked_sub(WINDOW)?;
    let (at, _) = *rows.get(end - 1)?;
    let future = rows.get(end..)?;
    let targets: Vec<Option<f32>> = HORIZONS
        .iter()
        .flat_map(|horizon| target(future, at + horizon, horizon * 0.5))
        .collect();
    targets
        .iter()
        .any(Option::is_some)
        .then(|| (window(&rows[start..end]), targets))
}

fn target(future: &[(f64, Features)], when: f64, tolerance: f64) -> Vec<Option<f32>> {
    let found = future
        .iter()
        .find(|(at, _)| *at >= when)
        .filter(|(at, _)| *at <= when + tolerance);
    METRICS
        .iter()
        .map(|metric| {
            found
                .and_then(|(_, features)| features.get(*metric))
                .map(|value| value / SCALE)
        })
        .collect()
}
