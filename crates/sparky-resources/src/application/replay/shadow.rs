use std::time::{Duration, Instant};

use super::volatile::{Recorder, Volatile};
use crate::application::baseline::{ALPHA, BETA};
use crate::domain::{Activities, Activity, Authority, Identity};
use crate::orchestration::planner::STALL;
use crate::orchestration::{Conductor, Contention, Decision, Moment, Verdict};
use crate::policy::Capabilities;
use crate::predict::Holt;
use crate::safety::Ledger;
use crate::telemetry::consumers::Consumer;

const HOG: u32 = 900;
const CADENCE: u64 = 2;
const AHEAD: f64 = 10.0;
const RATE: u64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Observed,
    Confirmed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub stalls: Vec<f32>,
    pub transient: bool,
    pub protected_hog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub interventions: usize,
    pub latency: Option<u64>,
    pub false_positives: usize,
    pub finished: usize,
    pub protected_writes: usize,
}

pub fn replay(scenario: &Scenario, mode: Mode) -> Outcome {
    let (executor, storage) = (Recorder::default(), Volatile::default());
    let capabilities = Capabilities {
        restore_nice: false,
        io_priority: true,
        affinity: true,
    };
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), capabilities, vec![3]);
    let consumers = [(hog(), RATE)];
    let protected: &[u32] = if scenario.protected_hog { &[HOG] } else { &[] };
    let gaming: Activities = [Activity::Gaming].into_iter().collect();
    let mut holt = Holt::new(ALPHA, BETA);
    let start = Instant::now();
    let mut seen = Seen::default();
    for (tick, stall) in scenario.stalls.iter().enumerate() {
        let seconds = tick as u64 * CADENCE;
        holt.update(seconds as f64, *stall);
        let confirmed = holt.forecast(AHEAD).is_some_and(|ahead| ahead >= STALL);
        let open = mode == Mode::Observed || conductor.adjusting() || confirmed;
        conductor.tick(&Moment {
            at: start + Duration::from_secs(seconds),
            seconds,
            activities: gaming,
            contention: Contention {
                cpu_stall: Some(if open { *stall } else { stall.min(STALL - 1.0) }),
                io_stall: None,
            },
            consumers: &consumers,
            protected,
            authority: Authority::Adaptive,
            confidence: 1.0,
        });
        seen.absorb(&conductor);
    }
    measure(scenario, &seen, &executor)
}

#[derive(Default)]
struct Seen {
    recorded: u64,
    applied: Vec<u64>,
    finished: usize,
}

impl Seen {
    fn absorb(&mut self, conductor: &Conductor<&Recorder, &Volatile>) {
        let now = conductor.recorded();
        let entries = conductor.decisions();
        for decision in fresh(&entries, self.recorded, now) {
            match decision.verdict {
                Verdict::Applied => self.applied.push(decision.at_seconds),
                Verdict::Finished(_) | Verdict::Reverted(_) => self.finished += 1,
                _ => {}
            }
        }
        self.recorded = now;
    }
}

pub fn fresh(entries: &[Decision], before: u64, now: u64) -> &[Decision] {
    let count = usize::try_from(now.saturating_sub(before))
        .unwrap_or(usize::MAX)
        .min(entries.len());
    &entries[entries.len() - count..]
}

fn measure(scenario: &Scenario, seen: &Seen, executor: &Recorder) -> Outcome {
    let onset = scenario
        .stalls
        .iter()
        .position(|stall| *stall >= STALL)
        .map(|tick| tick as u64 * CADENCE);
    Outcome {
        interventions: seen.applied.len(),
        latency: onset
            .zip(seen.applied.first())
            .map(|(onset, first)| first - onset),
        false_positives: if scenario.transient {
            seen.applied.len()
        } else {
            0
        },
        finished: seen.finished,
        protected_writes: match scenario.protected_hog {
            true => executor.writes_to(HOG),
            false => 0,
        },
    }
}

fn hog() -> Consumer {
    Consumer {
        identity: Identity {
            pid: HOG,
            started: 1,
        },
        parent: 1,
        name: "rustc".to_owned(),
        ticks: 0,
    }
}
