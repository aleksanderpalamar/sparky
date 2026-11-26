use super::dense::Dense;
use super::gru::{Gru, Step};
use super::objective::{Objective, measure};
use super::params::Parameters;
use super::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub steps: usize,
    pub context: usize,
    pub memory: usize,
    pub hidden: usize,
    pub outputs: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hybrid {
    pub params: Parameters,
    gru: Gru,
    blend: Dense,
    head: Dense,
}

struct Pass {
    steps: Vec<Step>,
    joined: Vec<f32>,
    active: Vec<f32>,
    output: Vec<f32>,
}

impl Hybrid {
    pub fn new(shape: Shape, seed: u64) -> Self {
        let mut rng = Rng::seeded(seed);
        let mut params = Parameters::default();
        let gru = Gru::new(&mut params, shape.steps, shape.memory, &mut rng);
        let blend = Dense::new(
            &mut params,
            shape.memory + shape.context,
            shape.hidden,
            &mut rng,
        );
        let head = Dense::new(&mut params, shape.hidden, shape.outputs, &mut rng);
        Self {
            params,
            gru,
            blend,
            head,
        }
    }

    pub fn still(shape: Shape, seed: u64) -> Self {
        let mut model = Self::new(shape, seed);
        model.head.clear(&mut model.params);
        model
    }

    pub fn widen(&self, shape: Shape) -> Option<Hybrid> {
        let same = self.gru.inputs() == shape.steps
            && self.gru.hidden == shape.memory
            && self.blend.outputs == shape.hidden
            && self.head.outputs == shape.outputs
            && self.blend.inputs <= shape.memory + shape.context;
        if !same {
            return None;
        }
        let mut wider = Hybrid::new(shape, 0);
        self.gru
            .transplant(&self.params, &wider.gru, &mut wider.params);
        self.blend
            .transplant(&self.params, &wider.blend, &mut wider.params);
        self.head
            .transplant(&self.params, &wider.head, &mut wider.params);
        Some(wider)
    }

    pub fn settle(&mut self, decay: f32) {
        self.head
            .shrink(&mut self.params, 1.0 - decay.clamp(0.0, 1.0));
    }

    pub fn predict(&self, sequence: &[Vec<f32>], context: &[f32]) -> Vec<f32> {
        self.pass(sequence, context).output
    }

    pub fn loss(&self, sequence: &[Vec<f32>], context: &[f32], target: &[Option<f32>]) -> f32 {
        measure(Objective::Squared, &self.predict(sequence, context), target).0
    }

    pub fn accumulate(
        &mut self,
        sequence: &[Vec<f32>],
        context: &[f32],
        target: &[Option<f32>],
    ) -> f32 {
        self.accumulate_with(Objective::Squared, sequence, context, target)
    }

    pub fn accumulate_with(
        &mut self,
        objective: Objective,
        sequence: &[Vec<f32>],
        context: &[f32],
        target: &[Option<f32>],
    ) -> f32 {
        let pass = self.pass(sequence, context);
        let (loss, gradient) = measure(objective, &pass.output, target);
        let mut active = vec![0.0; self.blend.outputs];
        self.head
            .backward(&mut self.params, &pass.active, &gradient, Some(&mut active));
        for (delta, value) in active.iter_mut().zip(&pass.active) {
            *delta *= slope(*value);
        }
        let mut joined = vec![0.0; self.blend.inputs];
        self.blend
            .backward(&mut self.params, &pass.joined, &active, Some(&mut joined));
        joined.truncate(self.gru.hidden);
        let mut hidden = joined;
        for step in pass.steps.iter().rev() {
            hidden = self.gru.backward(&mut self.params, step, &hidden);
        }
        loss
    }

    fn pass(&self, sequence: &[Vec<f32>], context: &[f32]) -> Pass {
        let mut state = vec![0.0; self.gru.hidden];
        let mut steps = Vec::with_capacity(sequence.len());
        for input in sequence {
            let step = self.gru.step(&self.params, input, &state);
            state.clone_from(&step.output);
            steps.push(step);
        }
        let joined: Vec<f32> = state.into_iter().chain(context.iter().copied()).collect();
        let mut sums = vec![0.0; self.blend.outputs];
        self.blend.forward(&self.params, &joined, &mut sums);
        let active: Vec<f32> = sums.iter().map(|sum| activate(*sum)).collect();
        let mut output = vec![0.0; self.head.outputs];
        self.head.forward(&self.params, &active, &mut output);
        Pass {
            steps,
            joined,
            active,
            output,
        }
    }
}

pub(super) fn activate(sum: f32) -> f32 {
    sum.tanh()
}

pub(super) fn slope(active: f32) -> f32 {
    1.0 - active * active
}
