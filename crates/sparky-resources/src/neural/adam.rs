use super::params::Parameters;

const BETA1: f32 = 0.9;
const BETA2: f32 = 0.999;
const EPSILON: f32 = 1e-8;
const CLIP: f32 = 5.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Adam {
    rate: f32,
    first: Vec<f32>,
    second: Vec<f32>,
    steps: i32,
}

impl Adam {
    pub fn new(rate: f32, parameters: usize) -> Self {
        Self {
            rate,
            first: vec![0.0; parameters],
            second: vec![0.0; parameters],
            steps: 0,
        }
    }

    pub fn step(&mut self, params: &mut Parameters) {
        let norm = params
            .grads
            .iter()
            .map(|grad| grad * grad)
            .sum::<f32>()
            .sqrt();
        let scale = if norm > CLIP { CLIP / norm } else { 1.0 };
        self.steps = self.steps.saturating_add(1);
        let first_fix = 1.0 - BETA1.powi(self.steps);
        let second_fix = 1.0 - BETA2.powi(self.steps);
        for index in 0..params.len().min(self.first.len()) {
            let grad = params.grads[index] * scale;
            self.first[index] = BETA1 * self.first[index] + (1.0 - BETA1) * grad;
            self.second[index] = BETA2 * self.second[index] + (1.0 - BETA2) * grad * grad;
            let first = self.first[index] / first_fix;
            let second = self.second[index] / second_fix;
            params.values[index] -= self.rate * first / (second.sqrt() + EPSILON);
        }
        params.zero_grads();
    }
}
