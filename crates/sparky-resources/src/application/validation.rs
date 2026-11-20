use super::diagnosis::{Phase, Tally};
use super::encoding::{HORIZONS, Sample, WINDOW, absolute, decode, example_at};
use crate::domain::METRICS;
use crate::neural::Hybrid;
use crate::predict::Holt;

const ALPHA: f32 = 0.2;
const BETA: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Errors {
    pub statistical: f64,
    pub neural: Option<f64>,
    pub count: usize,
}

pub fn validate(rows: &[Sample], from: usize, model: Option<&Hybrid>) -> Errors {
    tally(rows, from, model).errors(model.is_some())
}

pub fn tally(rows: &[Sample], from: usize, model: Option<&Hybrid>) -> Tally {
    let mut holts = [Holt::new(ALPHA, BETA); METRICS.len()];
    let mut phase = Phase::default();
    let mut tally = Tally::default();
    for end in 1..=rows.len() {
        let now = &rows[end - 1];
        for (slot, metric) in METRICS.iter().enumerate() {
            if let Some(value) = now.features.get(*metric) {
                holts[slot].update(now.at, value);
            }
        }
        let segment = phase.segment(now);
        if end < from.max(WINDOW) {
            continue;
        }
        let Some(example) = example_at(rows, end) else {
            continue;
        };
        let predicted = model.map(|model| {
            decode(
                &model.predict(&example.window, &example.context),
                &now.features,
            )
        });
        for (output, target) in absolute(&example.targets, &now.features).iter().enumerate() {
            let Some(target) = *target else {
                continue;
            };
            let (slot, horizon) = (output % METRICS.len(), HORIZONS[output / METRICS.len()]);
            let estimate = holts[slot]
                .forecast(horizon)
                .unwrap_or(0.0)
                .clamp(0.0, 100.0);
            let neural = predicted
                .as_ref()
                .and_then(|values| values.get(output))
                .map_or(0.0, |value| f64::from((value - target).abs()));
            tally.add(
                segment,
                output,
                neural,
                f64::from((estimate - target).abs()),
            );
        }
    }
    tally
}
