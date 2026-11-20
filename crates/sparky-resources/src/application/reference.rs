use super::baseline::{ALPHA, BETA, Insight};
use super::encoding::{HORIZONS, OUTPUTS, Sample};
use crate::domain::METRICS;
use crate::predict::Holt;

pub type Base = [Option<f32>; OUTPUTS];

pub fn bases(rows: &[Sample]) -> Vec<Base> {
    let mut holts = [Holt::new(ALPHA, BETA); METRICS.len()];
    rows.iter()
        .map(|sample| {
            for (slot, metric) in METRICS.iter().enumerate() {
                if let Some(value) = sample.features.get(*metric) {
                    holts[slot].update(sample.at, value);
                }
            }
            std::array::from_fn(|output| {
                let horizon = HORIZONS.get(output / METRICS.len())?;
                holts[output % METRICS.len()]
                    .forecast(*horizon)
                    .map(|value| value.clamp(0.0, 100.0))
            })
        })
        .collect()
}

pub fn base(insights: &[Insight]) -> Base {
    std::array::from_fn(|output| {
        let insight = insights.get(output % METRICS.len())?;
        match output / METRICS.len() {
            0 => insight.estimate_short,
            _ => insight.estimate_long,
        }
    })
}
