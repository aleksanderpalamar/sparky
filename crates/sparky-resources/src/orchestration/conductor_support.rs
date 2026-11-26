use std::time::{Duration, Instant};

use super::conductor::{Conductor, Moment};
use super::log::Verdict;
use super::planner::Contention;
use crate::domain::{Activities, Activity, Authority, Change, ChangeKind, Identity};
use crate::policy::Capabilities;
use crate::safety::fake_executor::FakeExecutor;
use crate::telemetry::consumers::{Consumer, Usage};

pub const NO_NICE: Capabilities = Capabilities {
    restore_nice: false,
    io_priority: true,
    affinity: true,
};

pub fn hog() -> Vec<(Consumer, Usage)> {
    vec![(
        Consumer {
            identity: Identity {
                pid: 900,
                started: 1,
            },
            parent: 1,
            name: "rustc".to_owned(),
            ticks: 0,
        },
        Usage::cpu(500),
    )]
}

pub fn gaming() -> Activities {
    [Activity::Gaming].into_iter().collect()
}

pub fn moment<'a>(
    start: Instant,
    seconds: u64,
    activities: Activities,
    stall: f32,
    authority: Authority,
    consumers: &'a [(Consumer, Usage)],
) -> Moment<'a> {
    Moment {
        at: start + Duration::from_secs(seconds),
        seconds,
        activities,
        contention: Contention {
            cpu_stall: Some(stall),
            io_stall: None,
        },
        consumers,
        protected: &[],
        authority,
        confidence: 1.0,
    }
}

pub fn affinity(executor: &FakeExecutor) -> Option<Change> {
    executor
        .values
        .borrow()
        .get(&(900, ChangeKind::Affinity))
        .cloned()
}

pub fn verdicts<E: crate::ports::Executor, S: crate::ports::Storage>(
    conductor: &Conductor<E, S>,
) -> Vec<Verdict> {
    conductor
        .decisions()
        .into_iter()
        .map(|decision| decision.verdict)
        .collect()
}
