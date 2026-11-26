use std::time::{Duration, Instant};

use crate::application::archive::GAP;
use crate::application::encoding::Sample;
use crate::application::learning::{Learner, Plan, Verdict};
use crate::neural::Hybrid;

const SLICE: Duration = Duration::from_millis(200);

pub fn chain(parts: Vec<Vec<Sample>>) -> Vec<Sample> {
    let mut rows: Vec<Sample> = Vec::new();
    for part in parts {
        let (Some(first), Some(last)) = (part.first(), rows.last()) else {
            rows.extend(part);
            continue;
        };
        let shift = last.at + GAP - first.at;
        rows.extend(part.into_iter().map(|sample| Sample {
            at: sample.at + shift,
            ..sample
        }));
    }
    rows
}

pub fn freeze(plan: Plan, training: &[Sample]) -> (Option<Hybrid>, Verdict) {
    let mut learner = Learner::new(plan);
    learner.resume(training.to_vec(), None);
    loop {
        match learner.step(None, Instant::now() + SLICE) {
            Verdict::Training => continue,
            Verdict::Promoted(model, errors) => {
                return (Some((*model).clone()), Verdict::Promoted(model, errors));
            }
            other => return (learner.incumbent().cloned(), other),
        }
    }
}
