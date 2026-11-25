use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::domain::{Adjustment, Authority, Change, ChangeKind, IoClass};

const MIN_CONFIDENCE: f32 = 0.8;
const COOLDOWN: Duration = Duration::from_secs(60);
const WINDOW: Duration = Duration::from_secs(60);
const PER_WINDOW: usize = 3;
const LOWEST_PRIORITY: i8 = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Block {
    NoAuthority,
    TelemetryMissing,
    LowConfidence,
    Protected,
    OutOfRange,
    NotReversible,
    Conservative,
    Cooldown,
    RateLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities {
    pub restore_nice: bool,
    pub io_priority: bool,
    pub affinity: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Situation<'a> {
    pub authority: Authority,
    pub telemetry: bool,
    pub protected: &'a [u32],
    pub capabilities: Capabilities,
    pub now: Instant,
}

#[derive(Default)]
pub struct Policy {
    accepted: VecDeque<(Instant, u32)>,
}

impl Policy {
    pub fn decide(
        &mut self,
        adjustment: &Adjustment,
        confidence: f32,
        situation: &Situation,
    ) -> Result<(), Block> {
        static_rules(adjustment, confidence, situation)?;
        self.accepted
            .retain(|(at, _)| situation.now.duration_since(*at) < WINDOW.max(COOLDOWN));
        let pid = adjustment.target.pid;
        if self
            .accepted
            .iter()
            .any(|(at, target)| *target == pid && situation.now.duration_since(*at) < COOLDOWN)
        {
            return Err(Block::Cooldown);
        }
        if self
            .accepted
            .iter()
            .filter(|(at, _)| situation.now.duration_since(*at) < WINDOW)
            .count()
            >= PER_WINDOW
        {
            return Err(Block::RateLimit);
        }
        self.accepted.push_back((situation.now, pid));
        Ok(())
    }
}

pub fn permits(authority: Authority, kind: ChangeKind) -> bool {
    match authority {
        Authority::None => false,
        Authority::Conservative => kind != ChangeKind::Affinity,
        Authority::Adaptive => true,
    }
}

fn static_rules(
    adjustment: &Adjustment,
    confidence: f32,
    situation: &Situation,
) -> Result<(), Block> {
    if situation.authority == Authority::None {
        return Err(Block::NoAuthority);
    }
    if !situation.telemetry {
        return Err(Block::TelemetryMissing);
    }
    if confidence.is_nan() || confidence < MIN_CONFIDENCE {
        return Err(Block::LowConfidence);
    }
    if adjustment.target.pid <= 1 || situation.protected.contains(&adjustment.target.pid) {
        return Err(Block::Protected);
    }
    if !in_range(&adjustment.change) {
        return Err(Block::OutOfRange);
    }
    if !reversible(adjustment.change.kind(), situation.capabilities) {
        return Err(Block::NotReversible);
    }
    if !permits(situation.authority, adjustment.change.kind()) {
        return Err(Block::Conservative);
    }
    Ok(())
}

fn in_range(change: &Change) -> bool {
    match change {
        Change::Nice(value) => (0..=LOWEST_PRIORITY).contains(value),
        Change::IoPriority(IoClass::BestEffort(level)) => *level <= 7,
        Change::IoPriority(_) => true,
        Change::Affinity(cpus) => !cpus.is_empty(),
    }
}

fn reversible(kind: ChangeKind, capabilities: Capabilities) -> bool {
    match kind {
        ChangeKind::Nice => capabilities.restore_nice,
        ChangeKind::IoPriority => capabilities.io_priority,
        ChangeKind::Affinity => capabilities.affinity,
    }
}
