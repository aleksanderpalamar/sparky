use super::archive::GAP;
use super::baseline::{ALPHA, BETA, Insight};
use super::encoding::{HORIZONS, OUTPUTS, SCALE, Sample};
use crate::domain::METRICS;
use crate::predict::Holt;

pub type Base = [Option<f32>; OUTPUTS];
pub type Misses = [Option<f32>; OUTPUTS];

pub fn continuous(previous: &Sample, next: &Sample) -> bool {
    next.at - previous.at < GAP
}

pub fn bases(rows: &[Sample]) -> Vec<Base> {
    let mut holts = [Holt::new(ALPHA, BETA); METRICS.len()];
    let mut previous: Option<&Sample> = None;
    rows.iter()
        .map(|sample| {
            if previous.is_some_and(|previous| !continuous(previous, sample)) {
                holts = [Holt::new(ALPHA, BETA); METRICS.len()];
            }
            previous = Some(sample);
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

pub fn misses(samples: &[Sample], bases: &[Base]) -> Misses {
    let Some(now) = samples.last() else {
        return [None; OUTPUTS];
    };
    std::array::from_fn(|output| {
        let horizon = *HORIZONS.get(output / METRICS.len())?;
        let metric = METRICS.get(output % METRICS.len())?;
        let when = now.at - horizon;
        let index = samples
            .partition_point(|sample| sample.at <= when)
            .checked_sub(1)?;
        samples
            .get(index)
            .filter(|then| when - then.at <= horizon * 0.5)?;
        let expected = bases.get(index)?.get(output).copied().flatten()?;
        let actual = now.features.get(*metric)?;
        Some((actual - expected) / SCALE)
    })
}
