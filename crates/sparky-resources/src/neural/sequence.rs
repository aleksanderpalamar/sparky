use super::dense::Dense;
use super::gru::{Gru, Step};
use super::params::Parameters;
use super::rng::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceModel {
    pub params: Parameters,
    gru: Gru,
    head: Dense,
}

impl SequenceModel {
    pub fn new(inputs: usize, hidden: usize, outputs: usize, seed: u64) -> Self {
        let mut rng = Rng::seeded(seed);
        let mut params = Parameters::default();
        let gru = Gru::new(&mut params, inputs, hidden, &mut rng);
        let head = Dense::new(&mut params, hidden, outputs, &mut rng);
        Self { params, gru, head }
    }

    pub fn still(inputs: usize, hidden: usize, outputs: usize, seed: u64) -> Self {
        let mut model = Self::new(inputs, hidden, outputs, seed);
        model.head.clear(&mut model.params);
        model
    }

    pub fn predict(&self, sequence: &[Vec<f32>]) -> Vec<f32> {
        let steps = self.unroll(sequence);
        self.project(&steps)
    }

    pub fn loss(&self, sequence: &[Vec<f32>], target: &[Option<f32>]) -> f32 {
        squared(&self.predict(sequence), target).0
    }

    pub fn accumulate(&mut self, sequence: &[Vec<f32>], target: &[Option<f32>]) -> f32 {
        let steps = self.unroll(sequence);
        let (loss, gradient) = squared(&self.project(&steps), target);
        let last = steps
            .last()
            .map_or_else(|| vec![0.0; self.gru.hidden], |step| step.output.clone());
        let mut hidden = vec![0.0; self.gru.hidden];
        self.head
            .backward(&mut self.params, &last, &gradient, Some(&mut hidden));
        for step in steps.iter().rev() {
            hidden = self.gru.backward(&mut self.params, step, &hidden);
        }
        loss
    }

    fn unroll(&self, sequence: &[Vec<f32>]) -> Vec<Step> {
        let mut state = vec![0.0; self.gru.hidden];
        let mut steps = Vec::with_capacity(sequence.len());
        for input in sequence {
            let step = self.gru.step(&self.params, input, &state);
            state.clone_from(&step.output);
            steps.push(step);
        }
        steps
    }

    fn project(&self, steps: &[Step]) -> Vec<f32> {
        let last = steps
            .last()
            .map_or_else(|| vec![0.0; self.gru.hidden], |step| step.output.clone());
        let mut output = vec![0.0; self.head.outputs];
        self.head.forward(&self.params, &last, &mut output);
        output
    }
}

fn squared(predicted: &[f32], target: &[Option<f32>]) -> (f32, Vec<f32>) {
    let present = target.iter().flatten().count().max(1) as f32;
    let mut loss = 0.0;
    let gradient = predicted
        .iter()
        .zip(target)
        .map(|(value, wanted)| {
            let Some(wanted) = wanted else {
                return 0.0;
            };
            let error = value - wanted;
            loss += error * error / present;
            2.0 * error / present
        })
        .collect();
    (loss, gradient)
}
