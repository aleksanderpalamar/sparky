use std::time::{Duration, Instant};

use crate::application::encoding::{HORIZONS, OUTPUTS, STEP_INPUTS, WINDOW, decode, window};
use crate::application::learning::{Learner, Plan, Verdict};
use crate::application::{Baseline, Insight, Knowledge as Validity, Observation};
use crate::domain::{Features, TrainingState};
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
    recent: Vec<(f64, Features)>,
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
        self.learner.record(at, observation.features);
        self.recent.push((at, observation.features));
        if self.recent.len() > WINDOW {
            self.recent.remove(0);
        }
        let mut insights = self.baseline.observe(at, &observation.features);
        let neural = self
            .active
            .as_ref()
            .filter(|_| self.recent.len() == WINDOW)
            .map(|model| decode(&model.predict(&window(&self.recent))));
        if let Some(values) = neural {
            for (slot, insight) in insights.iter_mut().enumerate() {
                insight.neural_short = values.get(slot).copied();
                insight.neural_long = values.get(slot + OUTPUTS / HORIZONS.len()).copied();
            }
        }
        insights
    }

    pub fn train(&mut self, training: TrainingState) -> bool {
        if training != TrainingState::Allowed && training != TrainingState::Active {
            return false;
        }
        match self
            .learner
            .step(self.active.as_ref(), Instant::now() + SLICE)
        {
            Verdict::Promoted(model, _) => self.promote(*model),
            Verdict::Training => return true,
            Verdict::Waiting | Verdict::Rejected(_) => {}
        }
        false
    }

    pub fn forget(&mut self) {
        self.active = None;
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
