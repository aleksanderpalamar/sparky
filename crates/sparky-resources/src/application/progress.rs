use serde::{Deserialize, Serialize};

use super::validation::Errors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Promoted,
    BehindStatistics,
    BehindActive,
    Unstable,
    NoData,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub at_seconds: u64,
    pub outcome: Outcome,
    pub neural: Option<f64>,
    pub statistical: f64,
}

pub const REMEMBERED: usize = 16;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Progress {
    pub collected: usize,
    pub needed: usize,
    #[serde(default)]
    pub window: usize,
    pub epoch: usize,
    pub epochs: usize,
    pub rounds: u32,
    pub parameters: usize,
    pub history: Vec<Round>,
}

impl Progress {
    pub fn last(&self) -> Option<&Round> {
        self.history.last()
    }
}

const TOLERANCE: f64 = 1e-9;

pub fn outcome(margin: f64, errors: &Errors, current: Option<f64>, finite: bool) -> Outcome {
    let Some(score) = errors.neural.filter(|_| errors.count > 0) else {
        return Outcome::NoData;
    };
    if !finite || !score.is_finite() {
        return Outcome::Unstable;
    }
    if !beats(score, errors.statistical, margin) {
        return Outcome::BehindStatistics;
    }
    if current.is_some_and(|current| score >= current) {
        return Outcome::BehindActive;
    }
    Outcome::Promoted
}

fn beats(score: f64, statistical: f64, margin: f64) -> bool {
    let limit = statistical * (1.0 - margin);
    statistical > 0.0 && score <= limit * (1.0 + TOLERANCE)
}

impl Round {
    pub fn new(at_seconds: u64, outcome: Outcome, errors: &Errors) -> Self {
        Self {
            at_seconds,
            outcome,
            neural: errors.neural,
            statistical: errors.statistical,
        }
    }
}
