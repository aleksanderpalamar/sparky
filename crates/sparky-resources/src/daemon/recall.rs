use crate::application::archive::{unpack_rounds, unpack_samples};
use crate::application::encoding::Sample;
use crate::application::learning::{Plan, shape};
use crate::application::progress::Round;
use crate::neural::Hybrid;
use crate::persistence::{Checkpoint, Kind, Knowledge, Restored};
use crate::ports::Storage;

pub const REVISION: u64 = 2;

pub fn owner(kind: Kind, hardware: u64) -> u64 {
    match kind {
        Kind::Sequence | Kind::Candidate => hardware ^ (REVISION << 56),
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
        let values = self.values(kind)?;
        let mut model = Hybrid::new(shape(plan), 0);
        (values.len() == model.params.len()).then(|| {
            model.params.values = values;
            model
        })
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

    pub fn forget(&mut self) {
        if let Some(Err(error)) = self.knowledge.as_mut().map(Knowledge::forget) {
            eprintln!("sparky-resources: conhecimento não apagado: {error}");
        }
    }

    fn values(&self, kind: Kind) -> Option<Vec<f32>> {
        match self
            .knowledge
            .as_ref()?
            .restore(kind, owner(kind, self.hardware))
        {
            Restored::Model(checkpoint) => Some(checkpoint.values),
            _ => None,
        }
    }
}
