use std::time::{Duration, Instant};

use super::encoding::Sample;
use super::learning::{Learner, Plan, Verdict, shape};
use crate::domain::{Activities, Features, Metric};
use crate::neural::Hybrid;

pub fn small() -> Plan {
    Plan {
        capacity: 400,
        minimum: 300,
        epochs: 30,
        batch: 16,
        rate: 0.01,
        decay: 0.0,
        margin: 0.05,
        hidden: 8,
        blend: 8,
        seed: 1,
    }
}

pub fn sample(at: f64, value: f32) -> Sample {
    Sample {
        at,
        features: Features::default().with(Metric::CpuBusy, Some(value)),
        activities: Activities::default(),
    }
}

pub fn wave(learner: &mut Learner, count: usize) {
    for index in 0..count {
        let at = index as f64 * 2.0;
        let value = 50.0 + 30.0 * ((at / 40.0) * std::f64::consts::TAU).sin() as f32;
        learner.record(sample(at, value));
    }
}

pub fn finish(learner: &mut Learner) -> Verdict {
    for _ in 0..10_000 {
        match learner.step(None, Instant::now() + Duration::from_millis(50)) {
            Verdict::Training => continue,
            other => return other,
        }
    }
    Verdict::Training
}

pub fn fresh(plan: Plan) -> Hybrid {
    Hybrid::still(shape(&plan), plan.seed)
}

pub fn stubborn() -> Plan {
    Plan {
        margin: 1.0,
        epochs: 2,
        ..small()
    }
}
