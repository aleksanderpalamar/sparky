use super::params::{Parameters, Slot};
use super::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dense {
    weights: Slot,
    bias: Slot,
    pub inputs: usize,
    pub outputs: usize,
}

impl Dense {
    pub fn new(params: &mut Parameters, inputs: usize, outputs: usize, rng: &mut Rng) -> Self {
        let weights = params.allocate(inputs * outputs);
        let bias = params.allocate(outputs);
        params.randomize(weights, inputs, outputs, rng);
        Self {
            weights,
            bias,
            inputs,
            outputs,
        }
    }

    pub fn clear(&self, params: &mut Parameters) {
        self.shrink(params, 0.0);
    }

    pub fn shrink(&self, params: &mut Parameters, keep: f32) {
        for slot in [self.weights, self.bias] {
            for value in &mut params.values[slot.range()] {
                *value *= keep;
            }
        }
    }

    pub fn forward(&self, params: &Parameters, input: &[f32], output: &mut [f32]) {
        let weights = &params.values[self.weights.range()];
        let bias = &params.values[self.bias.range()];
        for (index, out) in output.iter_mut().enumerate().take(self.outputs) {
            let row = &weights[index * self.inputs..(index + 1) * self.inputs];
            *out = bias[index]
                + row
                    .iter()
                    .zip(input)
                    .map(|(weight, value)| weight * value)
                    .sum::<f32>();
        }
    }

    pub fn backward(
        &self,
        params: &mut Parameters,
        input: &[f32],
        gradient: &[f32],
        input_gradient: Option<&mut [f32]>,
    ) {
        let (values, grads) = (&params.values, &mut params.grads);
        for (index, delta) in gradient.iter().enumerate().take(self.outputs) {
            let row = self.weights.offset + index * self.inputs;
            for (column, value) in input.iter().enumerate().take(self.inputs) {
                grads[row + column] += delta * value;
            }
            grads[self.bias.offset + index] += delta;
        }
        let Some(input_gradient) = input_gradient else {
            return;
        };
        for (column, slot) in input_gradient.iter_mut().enumerate().take(self.inputs) {
            *slot += gradient
                .iter()
                .enumerate()
                .map(|(index, delta)| {
                    values[self.weights.offset + index * self.inputs + column] * delta
                })
                .sum::<f32>();
        }
    }
}

pub fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}
