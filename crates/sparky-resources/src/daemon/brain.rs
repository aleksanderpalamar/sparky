use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::recall::Recall;
use crate::application::archive::{pack_rounds, pack_samples};
use crate::application::encoding::{HORIZONS, OUTPUTS, Sample, context, decode};
use crate::application::learning::{Learner, Plan, Verdict};
use crate::application::progress::{Outcome, REMEMBERED};
use crate::application::recent::Recent;
use crate::application::reference::base;
use crate::application::{Baseline, Insight, Knowledge as Validity, Observation, Progress, Round};
use crate::domain::TrainingState;
use crate::neural::Hybrid;
use crate::persistence::{Kind, Knowledge};
use crate::ports::Storage;

const SLICE: Duration = Duration::from_millis(40);
const KEEP_EVERY: f64 = 300.0;

pub struct Brain<S> {
    baseline: Baseline,
    learner: Learner,
    recall: Recall<S>,
    active: Option<Hybrid>,
    recent: Recent,
    rounds: u32,
    history: VecDeque<Round>,
    kept_at: Option<f64>,
}

impl<S: Storage> Brain<S> {
    pub fn new(knowledge: Option<Knowledge<S>>, hardware: u64) -> Self {
        let plan = Plan::default();
        let recall = Recall::new(knowledge, hardware);
        let mut learner = Learner::new(plan);
        let candidate = recall
            .model(Kind::Candidate, &plan)
            .or_else(|| recall.inherited(Kind::Candidate, &plan))
            .or_else(|| recall.inherited(Kind::Sequence, &plan));
        learner.resume(recall.samples(0.0), candidate);
        let (rounds, history) = recall.rounds().unwrap_or_default();
        let skip = history.len().saturating_sub(REMEMBERED);
        Self {
            baseline: Baseline::default(),
            learner,
            active: recall.model(Kind::Sequence, &plan),
            recall,
            recent: Recent::default(),
            rounds,
            history: history.into_iter().skip(skip).collect(),
            kept_at: None,
        }
    }

    pub fn validity(&self) -> Validity {
        match self.active {
            Some(_) => Validity::Validated,
            None => Validity::Unvalidated,
        }
    }

    pub fn written(&self) -> u64 {
        self.recall.written()
    }

    pub fn has_candidate(&self) -> bool {
        self.learner.incumbent().is_some()
    }

    pub fn insights(&mut self, at: f64, observation: &Observation) -> Vec<Insight> {
        let sample = Sample {
            at,
            features: observation.features,
            activities: observation.activities,
        };
        self.learner.record(sample);
        self.keep_collection(at);
        let mut insights = self.baseline.observe(at, &observation.features);
        let base = base(&insights);
        self.recent.push(sample, base);
        let neural = self
            .active
            .as_ref()
            .zip(self.recent.window())
            .map(|(model, window)| {
                let hints = context(
                    observation.activities,
                    &base,
                    &observation.features,
                    &self.recent.misses(),
                );
                decode(
                    &model.predict(&window, &hints),
                    &base,
                    &observation.features,
                )
            });
        if let Some(values) = neural {
            for (slot, insight) in insights.iter_mut().enumerate() {
                insight.neural_short = finite(values.get(slot));
                insight.neural_long = finite(values.get(slot + OUTPUTS / HORIZONS.len()));
            }
        }
        insights
    }

    pub fn progress(&self) -> Progress {
        Progress {
            rounds: self.rounds,
            history: self.history.iter().copied().collect(),
            ..self.learner.progress()
        }
    }

    pub fn train(&mut self, training: TrainingState, at_seconds: u64) -> bool {
        if training != TrainingState::Allowed && training != TrainingState::Active {
            return false;
        }
        match self
            .learner
            .step(self.active.as_ref(), Instant::now() + SLICE)
        {
            Verdict::Promoted(model, errors) => {
                self.finished(Round::new(at_seconds, Outcome::Promoted, &errors));
                self.recall
                    .keep(Kind::Sequence, model.params.values.clone());
                self.active = Some(*model);
            }
            Verdict::Rejected(errors, outcome) => {
                self.finished(Round::new(at_seconds, outcome, &errors))
            }
            Verdict::Training => return true,
            Verdict::Waiting => {}
        }
        false
    }

    pub fn forget(&mut self) -> std::io::Result<()> {
        self.active = None;
        self.learner.forget();
        self.rounds = 0;
        self.history.clear();
        self.recall.forget()
    }

    fn finished(&mut self, round: Round) {
        self.rounds = self.rounds.saturating_add(1);
        if self.history.len() == REMEMBERED {
            self.history.pop_front();
        }
        self.history.push_back(round);
        self.report(round);
        let history: Vec<Round> = self.history.iter().copied().collect();
        self.recall
            .keep(Kind::Rounds, pack_rounds(self.rounds, &history));
        match self.learner.incumbent() {
            Some(model) => {
                let values = model.params.values.clone();
                self.recall.keep(Kind::Candidate, values);
            }
            None => self.recall.discard(Kind::Candidate),
        }
    }

    fn report(&self, round: Round) {
        let neural = round
            .neural
            .map_or_else(|| "-".to_owned(), |error| format!("{error:.2}"));
        eprintln!(
            "sparky-resources: treino {} terminou ({:?}): erro da rede {neural}, da estatística {:.2}",
            self.rounds, round.outcome, round.statistical
        );
        for line in self
            .learner
            .diagnosis()
            .map(|tally| tally.lines())
            .unwrap_or_default()
        {
            eprintln!(
                "sparky-resources: diagnóstico treino {}: {line}",
                self.rounds
            );
        }
    }

    fn keep_collection(&mut self, at: f64) {
        let due = self.kept_at.get_or_insert(at);
        if at - *due < KEEP_EVERY {
            return;
        }
        *due = at;
        let values = pack_samples(self.learner.samples());
        self.recall.keep(Kind::History, values);
    }
}

fn finite(value: Option<&f32>) -> Option<f32> {
    value.copied().filter(|value| value.is_finite())
}
