use std::collections::VecDeque;
use std::time::Instant;

use super::encoding::{OUTPUTS, STEP_INPUTS, WINDOW, example_at};
use super::progress::{Outcome, Progress, outcome};
use super::validation::{Errors, validate};
use crate::domain::Features;
use crate::neural::{Adam, SequenceModel};

const LEAK_GUARD: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    pub capacity: usize,
    pub minimum: usize,
    pub epochs: usize,
    pub batch: usize,
    pub rate: f32,
    pub margin: f64,
    pub hidden: usize,
    pub seed: u64,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            capacity: 7200,
            minimum: 600,
            epochs: 20,
            batch: 16,
            rate: 0.003,
            margin: 0.05,
            hidden: 32,
            seed: 20261118,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Waiting,
    Training,
    Promoted(Box<SequenceModel>, Errors),
    Rejected(Errors, Outcome),
}

struct Session {
    model: SequenceModel,
    adam: Adam,
    rows: Vec<(f64, Features)>,
    cursor: usize,
    epoch: usize,
}

pub struct Learner {
    plan: Plan,
    history: VecDeque<(f64, Features)>,
    session: Option<Session>,
}

impl Learner {
    pub fn new(plan: Plan) -> Self {
        Self {
            plan,
            history: VecDeque::with_capacity(plan.capacity),
            session: None,
        }
    }

    pub fn record(&mut self, at: f64, features: Features) {
        if self.history.len() == self.plan.capacity {
            self.history.pop_front();
        }
        self.history.push_back((at, features));
    }

    pub fn progress(&self) -> Progress {
        Progress {
            collected: self.history.len(),
            needed: self.plan.minimum,
            epoch: self.session.as_ref().map_or(0, |session| session.epoch),
            epochs: self.plan.epochs,
            ..Progress::default()
        }
    }

    pub fn step(&mut self, active: Option<&SequenceModel>, deadline: Instant) -> Verdict {
        if self.session.is_none() && self.history.len() < self.plan.minimum {
            return Verdict::Waiting;
        }
        let plan = self.plan;
        let session = self
            .session
            .get_or_insert_with(|| start(plan, &self.history, active));
        let split = session.rows.len() * 7 / 10;
        let last = split.saturating_sub(LEAK_GUARD).max(WINDOW);
        while Instant::now() < deadline && session.epoch < plan.epochs {
            for end in session.cursor..(session.cursor + plan.batch).min(last) {
                if let Some((window, targets)) = example_at(&session.rows, end) {
                    session.model.accumulate(&window, &targets);
                }
            }
            session.adam.step(&mut session.model.params);
            session.cursor += plan.batch;
            if session.cursor >= last {
                session.cursor = WINDOW;
                session.epoch += 1;
            }
        }
        if session.epoch < plan.epochs {
            return Verdict::Training;
        }
        let Some(session) = self.session.take() else {
            return Verdict::Waiting;
        };
        judge(plan, session, split, active)
    }
}

fn start(
    plan: Plan,
    history: &VecDeque<(f64, Features)>,
    active: Option<&SequenceModel>,
) -> Session {
    let model = active
        .cloned()
        .unwrap_or_else(|| SequenceModel::new(STEP_INPUTS, plan.hidden, OUTPUTS, plan.seed));
    let adam = Adam::new(plan.rate, model.params.len());
    Session {
        model,
        adam,
        rows: history.iter().copied().collect(),
        cursor: WINDOW,
        epoch: 0,
    }
}

fn judge(plan: Plan, session: Session, split: usize, active: Option<&SequenceModel>) -> Verdict {
    let candidate = validate(&session.rows, split, Some(&session.model));
    let current = active.and_then(|model| validate(&session.rows, split, Some(model)).neural);
    let finite = session.model.params.is_finite();
    match outcome(plan.margin, &candidate, current, finite) {
        Outcome::Promoted => Verdict::Promoted(Box::new(session.model), candidate),
        rejected => Verdict::Rejected(candidate, rejected),
    }
}
