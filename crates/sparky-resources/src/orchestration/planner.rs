use serde::{Deserialize, Serialize};

use crate::domain::{Activities, Activity, Adjustment, Change, IoClass};
use crate::telemetry::consumers::Consumer;

pub const STALL: f32 = 20.0;
const MIN_TICKS: u64 = 50;
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

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub purpose: Purpose,
    pub name: String,
    pub options: Vec<Adjustment>,
}

pub fn plan(
    activities: Activities,
    contention: Contention,
    consumers: &[(Consumer, u64)],
    protected: &[u32],
    spare: &[u32],
) -> Option<Candidate> {
    if !(activities.contains(Activity::Gaming) || activities.contains(Activity::Recording)) {
        return None;
    }
    let hog = consumers
        .iter()
        .filter(|(consumer, ticks)| {
            *ticks >= MIN_TICKS && !protected.contains(&consumer.identity.pid)
        })
        .max_by_key(|(_, ticks)| *ticks)
        .map(|(consumer, _)| consumer)?;
    let target = hog.identity;
    let adjust = |change| Adjustment { target, change };
    if contention.cpu_stall.is_some_and(|stall| stall >= STALL) {
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
    contention
        .io_stall
        .filter(|stall| *stall >= STALL)
        .map(|_| Candidate {
            purpose: Purpose::EaseIo,
            name: hog.name.clone(),
            options: vec![adjust(Change::IoPriority(IoClass::Idle))],
        })
}
