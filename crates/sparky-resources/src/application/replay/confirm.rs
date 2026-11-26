use std::collections::BTreeMap;
use std::ops::Range;

use crate::application::diagnosis::{Phase, Segment};
use crate::application::encoding::{Sample, WINDOW, absolute, decode, example_at};
use crate::application::reference::{bases, continuous};
use crate::domain::METRICS;
use crate::neural::Hybrid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paired {
    pub at: f64,
    pub neural: Option<f64>,
    pub statistical: f64,
    pub persistence: Option<f64>,
}

pub type Cuts = BTreeMap<(Segment, usize), Vec<Paired>>;

pub fn confirm(rows: &[Sample], reserved: Range<usize>, model: Option<&Hybrid>) -> Cuts {
    let bases = bases(rows);
    let mut phase = Phase::default();
    let mut cuts = Cuts::new();
    for end in 1..=rows.len() {
        let now = &rows[end - 1];
        if end > 1 && !continuous(&rows[end - 2], now) {
            phase = Phase::default();
        }
        let segment = phase.segment(now);
        if !reserved.contains(&(end - 1)) || end < WINDOW {
            continue;
        }
        let (Some(example), Some(base)) = (example_at(rows, &bases, end), bases.get(end - 1))
        else {
            continue;
        };
        let predicted =
            model.map(|model| decode(&model.predict(&example.window, &example.context), base));
        for (output, target) in absolute(&example.targets, base).iter().enumerate() {
            let (Some(target), Some(estimate)) = (*target, base[output]) else {
                continue;
            };
            let error = |value: f32| f64::from((value - target).abs());
            let metric = METRICS[output % METRICS.len()];
            cuts.entry((segment, output)).or_default().push(Paired {
                at: now.at,
                neural: predicted
                    .as_ref()
                    .and_then(|values| values.get(output).copied().flatten())
                    .map(error),
                statistical: error(estimate),
                persistence: now.features.get(metric).map(error),
            });
        }
    }
    cuts
}
