use super::encoding::{HORIZONS, WINDOW, decode, example_at};
use crate::domain::{Features, METRICS};
use crate::neural::SequenceModel;
use crate::predict::Holt;

const ALPHA: f32 = 0.2;
const BETA: f32 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Errors {
    pub statistical: f64,
    pub neural: Option<f64>,
    pub count: usize,
}

pub fn validate(rows: &[(f64, Features)], from: usize, model: Option<&SequenceModel>) -> Errors {
    let mut holts = [Holt::new(ALPHA, BETA); METRICS.len()];
    let (mut statistical, mut neural, mut count) = (0.0_f64, 0.0_f64, 0_usize);
    for end in 1..=rows.len() {
        let (at, features) = &rows[end - 1];
        for (slot, metric) in METRICS.iter().enumerate() {
            if let Some(value) = features.get(*metric) {
                holts[slot].update(*at, value);
            }
        }
        if end < from.max(WINDOW) {
            continue;
        }
        let Some((window, targets)) = example_at(rows, end) else {
            continue;
        };
        let predicted = model.map(|model| decode(&model.predict(&window)));
        for (output, target) in targets.iter().enumerate() {
            let Some(target) = target.map(|value| value * 100.0) else {
                continue;
            };
            let (slot, horizon) = (output % METRICS.len(), HORIZONS[output / METRICS.len()]);
            let estimate = holts[slot]
                .forecast(horizon)
                .unwrap_or(0.0)
                .clamp(0.0, 100.0);
            statistical += f64::from((estimate - target).abs());
            neural += predicted
                .as_ref()
                .map_or(0.0, |values| f64::from((values[output] - target).abs()));
            count += 1;
        }
    }
    let mean = |total: f64| total / count.max(1) as f64;
    Errors {
        statistical: mean(statistical),
        neural: model.map(|_| mean(neural)),
        count,
    }
}
