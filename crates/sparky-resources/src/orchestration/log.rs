use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::planner::Purpose;
use crate::domain::{Activities, Change};
use crate::policy::Block;
use crate::safety::Outcome;

const CAPACITY: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "verdict", content = "detail")]
pub enum Verdict {
    Simulated,
    Applied,
    Blocked(Block),
    Failed,
    Kept,
    Reverted(Outcome),
    Finished(Outcome),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub at_seconds: u64,
    pub activities: Activities,
    pub purpose: Purpose,
    pub process: String,
    pub change: Change,
    pub stall: Option<f32>,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Default)]
pub struct Log {
    entries: VecDeque<Decision>,
    recorded: u64,
}

impl Log {
    pub fn push(&mut self, decision: Decision) {
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(decision);
        self.recorded += 1;
    }

    pub fn recorded(&self) -> u64 {
        self.recorded
    }

    pub fn entries(&self) -> Vec<Decision> {
        self.entries.iter().cloned().collect()
    }
}
