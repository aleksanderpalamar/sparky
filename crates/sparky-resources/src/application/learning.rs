use std::collections::VecDeque;
use std::time::Instant;

use super::diagnosis::Tally;
use super::encoding::{Example, Sample, WINDOW, example_at};
pub use super::plan::{Plan, shape};
use super::progress::{Outcome, Progress, outcome};
use super::validation::{Errors, tally, validate};
use crate::neural::{Adam, Hybrid};

const LEAK_GUARD: usize = 40;

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Waiting,
    Training,
    Promoted(Box<Hybrid>, Errors),
    Rejected(Errors, Outcome),
}

struct Session {
    model: Hybrid,
    adam: Adam,
    rows: Vec<Sample>,
    cursor: usize,
    epoch: usize,
}

pub struct Learner {
    plan: Plan,
    parameters: usize,
    history: VecDeque<Sample>,
    session: Option<Session>,
    incumbent: Option<Hybrid>,
    diagnosis: Option<Tally>,
}

impl Learner {
    pub fn new(plan: Plan) -> Self {
        let parameters = Hybrid::new(shape(&plan), plan.seed).params.len();
        Self {
            plan,
            parameters,
            history: VecDeque::with_capacity(plan.capacity),
            session: None,
            incumbent: None,
            diagnosis: None,
        }
    }

    pub fn samples(&self) -> impl Iterator<Item = &Sample> {
        self.history.iter()
    }

    pub fn incumbent(&self) -> Option<&Hybrid> {
        self.incumbent.as_ref()
    }

    pub fn diagnosis(&self) -> Option<&Tally> {
        self.diagnosis.as_ref()
    }

    pub fn resume(&mut self, samples: Vec<Sample>, incumbent: Option<Hybrid>) {
        let skip = samples.len().saturating_sub(self.plan.capacity);
        self.history = samples.into_iter().skip(skip).collect();
        self.incumbent = incumbent.filter(|model| model.params.len() == self.parameters);
    }

    pub fn record(&mut self, sample: Sample) {
        if self.history.len() == self.plan.capacity {
            self.history.pop_front();
        }
        self.history.push_back(sample);
    }

    pub fn progress(&self) -> Progress {
        Progress {
            collected: self.history.len(),
            needed: self.plan.minimum,
            window: self.plan.capacity,
            epoch: self.session.as_ref().map_or(0, |session| session.epoch),
            epochs: self.plan.epochs,
            parameters: self.parameters,
            ..Progress::default()
        }
    }

    pub fn step(&mut self, active: Option<&Hybrid>, deadline: Instant) -> Verdict {
        if self.session.is_none() && self.history.len() < self.plan.minimum {
            return Verdict::Waiting;
        }
        let plan = self.plan;
        let session = self
            .session
            .get_or_insert_with(|| start(plan, &self.history, self.incumbent.as_ref().or(active)));
        let split = session.rows.len() * 7 / 10;
        let last = split.saturating_sub(LEAK_GUARD).max(WINDOW);
        while Instant::now() < deadline && session.epoch < plan.epochs {
            for end in session.cursor..(session.cursor + plan.batch).min(last) {
                if let Some(example) = lesson(&session.rows, split, end) {
                    session
                        .model
                        .accumulate(&example.window, &example.context, &example.targets);
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
        let diagnosis = tally(&session.rows, split, Some(&session.model));
        let (verdict, incumbent) = judge(plan, session, split, active, diagnosis.errors(true));
        self.incumbent = incumbent;
        self.diagnosis = Some(diagnosis);
        verdict
    }

    #[cfg(test)]
    pub fn starting_point(&self) -> Option<&Hybrid> {
        self.session.as_ref().map(|session| &session.model)
    }

    pub fn cancel(&mut self) {
        self.session = None;
        self.incumbent = None;
    }

    pub fn forget(&mut self) {
        self.cancel();
        self.history.clear();
        self.diagnosis = None;
    }
}

pub fn lesson(rows: &[Sample], split: usize, end: usize) -> Option<Example> {
    example_at(rows.get(..split)?, end)
}

fn start(plan: Plan, history: &VecDeque<Sample>, origin: Option<&Hybrid>) -> Session {
    let model = origin
        .cloned()
        .unwrap_or_else(|| Hybrid::still(shape(&plan), plan.seed));
    let adam = Adam::new(plan.rate, model.params.len());
    Session {
        model,
        adam,
        rows: history.iter().copied().collect(),
        cursor: WINDOW,
        epoch: 0,
    }
}

fn judge(
    plan: Plan,
    session: Session,
    split: usize,
    active: Option<&Hybrid>,
    candidate: Errors,
) -> (Verdict, Option<Hybrid>) {
    let current = active.and_then(|model| validate(&session.rows, split, Some(model)).neural);
    let finite = session.model.params.is_finite();
    match outcome(plan.margin, &candidate, current, finite) {
        Outcome::Promoted => (Verdict::Promoted(Box::new(session.model), candidate), None),
        Outcome::Unstable => (Verdict::Rejected(candidate, Outcome::Unstable), None),
        rejected => (Verdict::Rejected(candidate, rejected), Some(session.model)),
    }
}
