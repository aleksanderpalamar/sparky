use serde::{Deserialize, Serialize};

use crate::domain::{Activities, Activity, Adjustment, Change, IoClass};
use crate::telemetry::consumers::{Consumer, Usage};

pub const STALL: f32 = 20.0;
const MIN_TICKS: u64 = 50;
const MIN_BYTES: u64 = 8 << 20;
const BACKGROUND_NICE: i8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    EaseCpu,
    EaseIo,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Contention {
    pub cpu_stall: Option<f32>,
    pub io_stall: Option<f32>,
}

impl Contention {
    pub fn stall(&self, purpose: Purpose) -> Option<f32> {
        match purpose {
            Purpose::EaseCpu => self.cpu_stall,
            Purpose::EaseIo => self.io_stall,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub purpose: Purpose,
    pub name: String,
    pub options: Vec<Adjustment>,
}

pub fn plan(
    activities: Activities,
    contention: Contention,
    consumers: &[(Consumer, Usage)],
    protected: &[u32],
    spare: &[u32],
) -> Option<Candidate> {
    if !(activities.contains(Activity::Gaming) || activities.contains(Activity::Recording)) {
        return None;
    }
    let contended = |stall: Option<f32>| stall.is_some_and(|stall| stall >= STALL);
    if contended(contention.cpu_stall) {
        let hog = heaviest(consumers, protected, |usage| Some(usage.cpu), MIN_TICKS)?;
        let adjust = |change| Adjustment {
            target: hog.identity,
            change,
        };
        let mut options = vec![adjust(Change::Nice(BACKGROUND_NICE))];
        if !spare.is_empty() {
            options.push(adjust(Change::Affinity(spare.to_vec())));
        }
        return Some(Candidate {
            purpose: Purpose::EaseCpu,
            name: hog.name.clone(),
            options,
        });
    }
    if !contended(contention.io_stall) {
        return None;
    }
    let hog = heaviest(consumers, protected, |usage| usage.io, MIN_BYTES)?;
    Some(Candidate {
        purpose: Purpose::EaseIo,
        name: hog.name.clone(),
        options: vec![Adjustment {
            target: hog.identity,
            change: Change::IoPriority(IoClass::Idle),
        }],
    })
}

fn heaviest<'a>(
    consumers: &'a [(Consumer, Usage)],
    protected: &[u32],
    measure: impl Fn(&Usage) -> Option<u64>,
    minimum: u64,
) -> Option<&'a Consumer> {
    consumers
        .iter()
        .filter(|(consumer, _)| !protected.contains(&consumer.identity.pid))
        .filter_map(|(consumer, usage)| Some((consumer, measure(usage)?)))
        .filter(|(_, amount)| *amount >= minimum)
        .max_by_key(|(_, amount)| *amount)
        .map(|(consumer, _)| consumer)
}
