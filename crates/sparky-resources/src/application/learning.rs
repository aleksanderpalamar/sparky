use std::collections::VecDeque;
use std::time::Instant;

use super::diagnosis::Tally;
use super::drill::{Division, divide, practise};
use super::encoding::{Example, Sample, WINDOW, example_at};
pub use super::plan::{Plan, shape};
use super::progress::{Outcome, Progress};
use super::reference::{Base, bases};
use super::selection::{Judged, best, judge};
use super::validation::Errors;
use crate::neural::{Adam, Hybrid};

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
    bases: Vec<Base>,
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
        let Division { split, last } = divide(session.rows.len());
        while Instant::now() < deadline && session.epoch < plan.epochs {
            let lessons: Vec<Example> = (session.cursor..(session.cursor + plan.batch).min(last))
                .filter_map(|end| lesson(&session.rows, &session.bases, split, end))
                .collect();
            practise(&mut session.model, &mut session.adam, &lessons, plan.decay);
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
        let previous = self.incumbent.take();
        let (model, diagnosis) = best(previous.clone(), session.model, &session.rows, split);
        let errors = diagnosis.errors(true);
        self.diagnosis = Some(diagnosis);
        match judge(&plan, model, &session.rows, split, active, errors) {
            Judged::Promoted(model, errors) => Verdict::Promoted(Box::new(model), errors),
            Judged::Rejected(errors, Outcome::NoData, _) => {
                self.incumbent = previous;
                Verdict::Rejected(errors, Outcome::NoData)
            }
            Judged::Rejected(errors, outcome, incumbent) => {
                self.incumbent = incumbent;
                Verdict::Rejected(errors, outcome)
            }
        }
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

pub fn lesson(rows: &[Sample], bases: &[Base], split: usize, end: usize) -> Option<Example> {
    example_at(rows.get(..split)?, bases.get(..split)?, end)
}

fn start(plan: Plan, history: &VecDeque<Sample>, origin: Option<&Hybrid>) -> Session {
    let model = origin
        .cloned()
        .unwrap_or_else(|| Hybrid::still(shape(&plan), plan.seed));
    let adam = Adam::new(plan.rate, model.params.len());
    let rows: Vec<Sample> = history.iter().copied().collect();
    Session {
        model,
        adam,
        bases: bases(&rows),
        rows,
        cursor: WINDOW,
        epoch: 0,
    }
}
