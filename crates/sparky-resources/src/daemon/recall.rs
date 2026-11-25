use crate::application::archive::{unpack_rounds, unpack_samples};
use crate::application::encoding::Sample;
use crate::application::learning::{Plan, shape};
use crate::application::progress::Round;
use crate::domain::activity::ACTIVITIES;
use crate::neural::{Hybrid, Shape};
use crate::persistence::{Checkpoint, Kind, Knowledge, Restored};
use crate::ports::Storage;

pub const REVISION: u64 = 4;
pub const LEGACY: u64 = 2;

pub fn owner(kind: Kind, hardware: u64) -> u64 {
    stamp(kind, hardware, REVISION)
}

pub fn stamp(kind: Kind, hardware: u64, revision: u64) -> u64 {
    match kind {
        Kind::Sequence | Kind::Candidate => hardware ^ (revision << 56),
        _ => hardware,
    }
}

pub struct Recall<S> {
    knowledge: Option<Knowledge<S>>,
    hardware: u64,
}

impl<S: Storage> Recall<S> {
    pub fn new(knowledge: Option<Knowledge<S>>, hardware: u64) -> Self {
        Self {
            knowledge,
            hardware,
        }
    }

    pub fn written(&self) -> u64 {
        self.knowledge.as_ref().map_or(0, Knowledge::written)
    }

    pub fn model(&self, kind: Kind, plan: &Plan) -> Option<Hybrid> {
        let current = self.values_at(kind, owner(kind, self.hardware), fits(shape(plan)))?;
        filled(shape(plan), current)
    }

    pub fn inherited(&self, kind: Kind, plan: &Plan) -> Option<Hybrid> {
        let legacy = Shape {
            context: ACTIVITIES.len(),
            ..shape(plan)
        };
        let values = self.values_at(kind, stamp(kind, self.hardware, LEGACY), fits(legacy))?;
        let mut inherited = filled(legacy, values)?.widen(shape(plan))?;
        inherited.settle(1.0);
        Some(inherited)
    }

    pub fn samples(&self, before: f64) -> Vec<Sample> {
        self.values(Kind::History)
            .map_or_else(Vec::new, |values| unpack_samples(&values, before))
    }

    pub fn rounds(&self) -> Option<(u32, Vec<Round>)> {
        unpack_rounds(&self.values(Kind::Rounds)?)
    }

    pub fn keep(&mut self, kind: Kind, values: Vec<f32>) {
        let checkpoint = Checkpoint {
            kind,
            hardware: owner(kind, self.hardware),
            values,
        };
        if let Some(Err(error)) = self
            .knowledge
            .as_mut()
            .map(|knowledge| knowledge.store(&checkpoint))
        {
            eprintln!("sparky-resources: {kind:?} não gravado: {error}");
        }
    }

    pub fn discard(&mut self, kind: Kind) {
        if let Some(Err(error)) = self
            .knowledge
            .as_mut()
            .map(|knowledge| knowledge.discard(kind))
        {
            eprintln!("sparky-resources: {kind:?} não apagado: {error}");
        }
    }

    pub fn forget(&mut self) -> std::io::Result<()> {
        self.knowledge.as_mut().map_or(Ok(()), Knowledge::forget)
    }

    fn values(&self, kind: Kind) -> Option<Vec<f32>> {
        self.values_at(kind, owner(kind, self.hardware), |_| true)
    }

    fn values_at(
        &self,
        kind: Kind,
        owner: u64,
        accept: impl Fn(&Checkpoint) -> bool,
    ) -> Option<Vec<f32>> {
        match self.knowledge.as_ref()?.restore_with(kind, owner, accept) {
            Restored::Model(checkpoint) => Some(checkpoint.values),
            _ => None,
        }
    }
}

fn filled(shape: Shape, values: Vec<f32>) -> Option<Hybrid> {
    let mut model = Hybrid::new(shape, 0);
    (values.len() == model.params.len()).then(|| {
        model.params.values = values;
        model
    })
}

fn fits(shape: Shape) -> impl Fn(&Checkpoint) -> bool {
    let expected = Hybrid::new(shape, 0).params.len();
    move |checkpoint| {
        checkpoint.values.len() == expected
            && checkpoint.values.iter().all(|value| value.is_finite())
    }
}
