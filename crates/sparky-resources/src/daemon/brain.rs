use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::application::encoding::{
    HORIZONS, OUTPUTS, STEP_INPUTS, Sample, WINDOW, decode, window,
};
use crate::application::learning::{Learner, Plan, Verdict};
use crate::application::progress::{Outcome, REMEMBERED};
use crate::application::{Baseline, Insight, Knowledge as Validity, Observation, Progress, Round};
use crate::domain::TrainingState;
use crate::neural::SequenceModel;
use crate::persistence::{Checkpoint, Kind, Knowledge, Restored};
use crate::ports::Storage;

const SLICE: Duration = Duration::from_millis(40);

pub struct Brain<S> {
    baseline: Baseline,
    learner: Learner,
    knowledge: Option<Knowledge<S>>,
    hardware: u64,
    active: Option<SequenceModel>,
    recent: Vec<Sample>,
    rounds: u32,
    history: VecDeque<Round>,
}

impl<S: Storage> Brain<S> {
    pub fn new(knowledge: Option<Knowledge<S>>, hardware: u64) -> Self {
        let plan = Plan::default();
        let active = knowledge
            .as_ref()
            .and_then(|knowledge| restore(knowledge, hardware, plan.hidden));
        Self {
            baseline: Baseline::default(),
            learner: Learner::new(plan),
            knowledge,
            hardware,
            active,
            recent: Vec::new(),
            rounds: 0,
            history: VecDeque::with_capacity(REMEMBERED),
        }
    }

    pub fn validity(&self) -> Validity {
        match self.active {
            Some(_) => Validity::Validated,
            None => Validity::Unvalidated,
        }
    }

    pub fn written(&self) -> u64 {
        self.knowledge.as_ref().map_or(0, Knowledge::written)
    }

    pub fn insights(&mut self, at: f64, observation: &Observation) -> Vec<Insight> {
        let sample = Sample {
            at,
            features: observation.features,
            activities: observation.activities,
        };
        self.learner.record(sample);
        self.recent.push(sample);
        if self.recent.len() > WINDOW {
            self.recent.remove(0);
        }
        let mut insights = self.baseline.observe(at, &observation.features);
        let neural = self
            .active
            .as_ref()
            .filter(|_| self.recent.len() == WINDOW)
            .map(|model| decode(&model.predict(&window(&self.recent)), &observation.features));
        if let Some(values) = neural {
            for (slot, insight) in insights.iter_mut().enumerate() {
                insight.neural_short = values.get(slot).copied();
                insight.neural_long = values.get(slot + OUTPUTS / HORIZONS.len()).copied();
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
                self.promote(*model);
            }
            Verdict::Rejected(errors, outcome) => {
                self.finished(Round::new(at_seconds, outcome, &errors))
            }
            Verdict::Training => return true,
            Verdict::Waiting => {}
        }
        false
    }

    fn finished(&mut self, round: Round) {
        self.rounds = self.rounds.saturating_add(1);
        if self.history.len() == REMEMBERED {
            self.history.pop_front();
        }
        self.history.push_back(round);
        let neural = round
            .neural
            .map_or_else(|| "-".to_owned(), |error| format!("{error:.2}"));
        eprintln!(
            "sparky-resources: treino {} terminou ({:?}): erro da rede {neural}, da estatística {:.2}",
            self.rounds, round.outcome, round.statistical
        );
    }

    pub fn forget(&mut self) {
        self.active = None;
        self.learner.cancel();
        if let Some(Err(error)) = self.knowledge.as_mut().map(Knowledge::forget) {
            eprintln!("sparky-resources: conhecimento não apagado: {error}");
        }
    }

    fn promote(&mut self, model: SequenceModel) {
        let checkpoint = Checkpoint {
            kind: Kind::Sequence,
            hardware: self.hardware,
            values: model.params.values.clone(),
        };
        if let Some(Err(error)) = self
            .knowledge
            .as_mut()
            .map(|knowledge| knowledge.promote(&checkpoint))
        {
            eprintln!("sparky-resources: checkpoint não gravado: {error}");
        }
        self.active = Some(model);
    }
}

fn restore<S: Storage>(
    knowledge: &Knowledge<S>,
    hardware: u64,
    hidden: usize,
) -> Option<SequenceModel> {
    let Restored::Model(checkpoint) = knowledge.restore(Kind::Sequence, hardware) else {
        return None;
    };
    let mut model = SequenceModel::new(STEP_INPUTS, hidden, OUTPUTS, 0);
    (checkpoint.values.len() == model.params.len()).then(|| {
        model.params.values = checkpoint.values;
        model
    })
}
