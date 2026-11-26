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

pub struct Frozen {
    pub model: Option<Hybrid>,
    pub verdict: Verdict,
    pub updates: usize,
}

pub fn freeze(plan: Plan, training: &[Sample]) -> Frozen {
    let mut learner = Learner::new(plan);
    learner.resume(training.to_vec(), None);
    loop {
        let verdict = learner.step(None, Instant::now() + SLICE);
        let model = match &verdict {
            Verdict::Training => continue,
            Verdict::Promoted(model, _) => Some((**model).clone()),
            _ => learner.incumbent().cloned(),
        };
        return Frozen {
            model,
            verdict,
            updates: learner.updates(),
        };
    }
}
