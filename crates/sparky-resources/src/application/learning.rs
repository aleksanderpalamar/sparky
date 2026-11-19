use std::collections::VecDeque;
use std::time::Instant;

use super::encoding::{CONTEXT_INPUTS, Example, OUTPUTS, STEP_INPUTS, Sample, WINDOW, example_at};
use super::progress::{Outcome, Progress, outcome};
use super::validation::{Errors, validate};
use crate::neural::{Adam, Hybrid, Shape};

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
    pub blend: usize,
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
            blend: 32,
            seed: 20261118,
        }
    }
}

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
        }
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
        let (verdict, incumbent) = judge(plan, session, split, active);
        self.incumbent = incumbent;
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
}

pub fn shape(plan: &Plan) -> Shape {
    Shape {
        steps: STEP_INPUTS,
        context: CONTEXT_INPUTS,
        memory: plan.hidden,
        hidden: plan.blend,
        outputs: OUTPUTS,
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
) -> (Verdict, Option<Hybrid>) {
    let candidate = validate(&session.rows, split, Some(&session.model));
    let current = active.and_then(|model| validate(&session.rows, split, Some(model)).neural);
    let finite = session.model.params.is_finite();
    match outcome(plan.margin, &candidate, current, finite) {
        Outcome::Promoted => (Verdict::Promoted(Box::new(session.model), candidate), None),
        Outcome::Unstable => (Verdict::Rejected(candidate, Outcome::Unstable), None),
        rejected => (Verdict::Rejected(candidate, rejected), Some(session.model)),
    }
}
