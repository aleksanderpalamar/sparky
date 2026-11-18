use super::dense::{Dense, sigmoid};
use super::params::Parameters;
use super::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gru {
    update_input: Dense,
    update_hidden: Dense,
    reset_input: Dense,
    reset_hidden: Dense,
    candidate_input: Dense,
    candidate_hidden: Dense,
    pub hidden: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    input: Vec<f32>,
    previous: Vec<f32>,
    update: Vec<f32>,
    reset: Vec<f32>,
    candidate: Vec<f32>,
    recurrent: Vec<f32>,
    pub output: Vec<f32>,
}

impl Gru {
    pub fn new(params: &mut Parameters, inputs: usize, hidden: usize, rng: &mut Rng) -> Self {
        let mut layer = |size| Dense::new(params, size, hidden, rng);
        Self {
            update_input: layer(inputs),
            update_hidden: layer(hidden),
            reset_input: layer(inputs),
            reset_hidden: layer(hidden),
            candidate_input: layer(inputs),
            candidate_hidden: layer(hidden),
            hidden,
        }
    }

    pub fn step(&self, params: &Parameters, input: &[f32], previous: &[f32]) -> Step {
        let both = |from_input: &Dense, from_hidden: &Dense| {
            let (mut sum, mut recurrent) = (vec![0.0; self.hidden], vec![0.0; self.hidden]);
            from_input.forward(params, input, &mut sum);
            from_hidden.forward(params, previous, &mut recurrent);
            (sum, recurrent)
        };
        let (update_in, update_rec) = both(&self.update_input, &self.update_hidden);
        let (reset_in, reset_rec) = both(&self.reset_input, &self.reset_hidden);
        let (candidate_in, recurrent) = both(&self.candidate_input, &self.candidate_hidden);
        let update: Vec<f32> = update_in
            .iter()
            .zip(&update_rec)
            .map(|(a, b)| sigmoid(a + b))
            .collect();
        let reset: Vec<f32> = reset_in
            .iter()
            .zip(&reset_rec)
            .map(|(a, b)| sigmoid(a + b))
            .collect();
        let candidate: Vec<f32> = (0..self.hidden)
            .map(|i| (candidate_in[i] + reset[i] * recurrent[i]).tanh())
            .collect();
        let output = (0..self.hidden)
            .map(|i| (1.0 - update[i]) * candidate[i] + update[i] * previous[i])
            .collect();
        Step {
            input: input.to_vec(),
            previous: previous.to_vec(),
            update,
            reset,
            candidate,
            recurrent,
            output,
        }
    }

    pub fn backward(&self, params: &mut Parameters, step: &Step, gradient: &[f32]) -> Vec<f32> {
        let size = self.hidden;
        let update_sum: Vec<f32> = (0..size)
            .map(|i| {
                gradient[i]
                    * (step.previous[i] - step.candidate[i])
                    * step.update[i]
                    * (1.0 - step.update[i])
            })
            .collect();
        let candidate_sum: Vec<f32> = (0..size)
            .map(|i| {
                gradient[i] * (1.0 - step.update[i]) * (1.0 - step.candidate[i] * step.candidate[i])
            })
            .collect();
        let reset_sum: Vec<f32> = (0..size)
            .map(|i| candidate_sum[i] * step.recurrent[i] * step.reset[i] * (1.0 - step.reset[i]))
            .collect();
        let recurrent: Vec<f32> = (0..size)
            .map(|i| candidate_sum[i] * step.reset[i])
            .collect();
        let mut previous: Vec<f32> = (0..size).map(|i| gradient[i] * step.update[i]).collect();
        self.update_input
            .backward(params, &step.input, &update_sum, None);
        self.reset_input
            .backward(params, &step.input, &reset_sum, None);
        self.candidate_input
            .backward(params, &step.input, &candidate_sum, None);
        self.update_hidden
            .backward(params, &step.previous, &update_sum, Some(&mut previous));
        self.reset_hidden
            .backward(params, &step.previous, &reset_sum, Some(&mut previous));
        self.candidate_hidden
            .backward(params, &step.previous, &recurrent, Some(&mut previous));
        previous
    }
}
