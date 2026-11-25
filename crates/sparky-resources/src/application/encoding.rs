use super::reference::{Base, Misses, bases, misses};
use crate::domain::activity::ACTIVITIES;
use crate::domain::{Activities, Features, METRICS};

pub const WINDOW: usize = 16;
pub const HORIZONS: [f64; 2] = [10.0, 60.0];
pub const STEP_INPUTS: usize = METRICS.len() * 2 + 1;
pub const CONTEXT_INPUTS: usize = ACTIVITIES.len() + OUTPUTS * 2;
pub const OUTPUTS: usize = METRICS.len() * HORIZONS.len();
pub(super) const SCALE: f32 = 100.0;
const TIME_SCALE: f64 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Example {
    pub window: Vec<Vec<f32>>,
    pub context: Vec<f32>,
    pub targets: Vec<Option<f32>>,
}

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
    values
        .chain(masks)
        .chain(std::iter::once((elapsed / TIME_SCALE).min(10.0) as f32))
        .collect()
}

pub fn context(
    activities: Activities,
    base: &Base,
    current: &Features,
    missed: &Misses,
) -> Vec<f32> {
    let flags = ACTIVITIES
        .iter()
        .map(|activity| flag(activities.contains(*activity)));
    let expected = base
        .iter()
        .zip(METRICS.iter().cycle())
        .map(|(reference, metric)| {
            reference
                .zip(current.get(*metric))
                .map_or(0.0, |(reference, now)| (reference - now) / SCALE)
        });
    let missed = missed.iter().map(|miss| miss.unwrap_or(0.0));
    flags.chain(expected).chain(missed).collect()
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

pub fn decode(outputs: &[f32], base: &Base) -> Vec<Option<f32>> {
    outputs
        .iter()
        .zip(base)
        .map(|(correction, reference)| {
            reference.map(|start| (start + correction * SCALE).clamp(0.0, 100.0))
        })
        .collect()
}

pub fn absolute(targets: &[Option<f32>], base: &Base) -> Vec<Option<f32>> {
    targets
        .iter()
        .zip(base)
        .map(|(error, reference)| Some((*reference)? + (*error)? * SCALE))
        .collect()
}

pub fn examples(rows: &[Sample]) -> Vec<Example> {
    let bases = bases(rows);
    (WINDOW..rows.len())
        .filter_map(|end| example_at(rows, &bases, end))
        .collect()
}

pub fn example_at(rows: &[Sample], bases: &[Base], end: usize) -> Option<Example> {
    let start = end.checked_sub(WINDOW)?;
    let now = rows.get(end - 1)?;
    let base = bases.get(end - 1)?;
    let future = rows.get(end..)?;
    let targets: Vec<Option<f32>> = HORIZONS
        .iter()
        .enumerate()
        .flat_map(|(step, horizon)| {
            let found = arrival(future, now.at + horizon, horizon * 0.5);
            METRICS.iter().enumerate().map(move |(slot, metric)| {
                let later = found?.features.get(*metric)?;
                let reference = (*base.get(step * METRICS.len() + slot)?)?;
                Some((later - reference) / SCALE)
            })
        })
        .collect();
    let missed = misses(rows.get(..end)?, bases.get(..end)?);
    targets.iter().any(Option::is_some).then(|| Example {
        window: window(&rows[start..end]),
        context: context(now.activities, base, &now.features, &missed),
        targets,
    })
}

fn arrival(future: &[Sample], when: f64, tolerance: f64) -> Option<&Sample> {
    future
        .iter()
        .find(|sample| sample.at >= when)
        .filter(|sample| sample.at <= when + tolerance)
}

fn flag(on: bool) -> f32 {
    if on { 1.0 } else { 0.0 }
}
