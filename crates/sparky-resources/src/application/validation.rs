use super::diagnosis::{Phase, Tally};
use super::encoding::{Sample, WINDOW, absolute, decode, example_at};
use super::reference::bases;
use crate::neural::Hybrid;

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
    let bases = bases(rows);
    let mut phase = Phase::default();
    let mut tally = Tally::default();
    for end in 1..=rows.len() {
        let now = &rows[end - 1];
        let segment = phase.segment(now);
        if end < from.max(WINDOW) {
            continue;
        }
        let (Some(example), Some(base)) = (example_at(rows, &bases, end), bases.get(end - 1))
        else {
            continue;
        };
        let predicted = model.map(|model| {
            decode(
                &model.predict(&example.window, &example.context),
                base,
                &now.features,
            )
        });
        for (output, target) in absolute(&example.targets, base).iter().enumerate() {
            let (Some(target), Some(estimate)) = (*target, base[output]) else {
                continue;
            };
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
