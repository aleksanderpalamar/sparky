use super::dense::{Dense, sigmoid};
use super::params::Parameters;
use super::rng::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct Mlp {
    pub params: Parameters,
    layers: Vec<Dense>,
}

struct Pass {
    inputs: Vec<Vec<f32>>,
    sums: Vec<Vec<f32>>,
}

impl Mlp {
    pub fn new(sizes: &[usize], seed: u64) -> Self {
        let mut rng = Rng::seeded(seed);
        let mut params = Parameters::default();
        let layers = sizes
            .windows(2)
            .map(|pair| Dense::new(&mut params, pair[0], pair[1], &mut rng))
            .collect();
        Self { params, layers }
    }

    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        self.pass(input)
            .sums
            .last()
            .map_or_else(Vec::new, |logits| {
                logits.iter().map(|logit| sigmoid(*logit)).collect()
            })
    }

    pub fn loss(&self, input: &[f32], target: &[f32]) -> f32 {
        let pass = self.pass(input);
        pass.sums.last().map_or(0.0, |logits| {
            logits
                .iter()
                .zip(target)
                .map(|(logit, wanted)| cross_entropy(*logit, *wanted))
                .sum()
        })
    }

    pub fn accumulate(&mut self, input: &[f32], target: &[f32]) -> f32 {
        let pass = self.pass(input);
        let Some(logits) = pass.sums.last() else {
            return 0.0;
        };
        let loss = logits
            .iter()
            .zip(target)
            .map(|(logit, wanted)| cross_entropy(*logit, *wanted))
            .sum();
        let mut gradient: Vec<f32> = logits
            .iter()
            .zip(target)
            .map(|(logit, wanted)| sigmoid(*logit) - wanted)
            .collect();
        for (index, layer) in self.layers.iter().enumerate().rev() {
            let mut below = vec![0.0; layer.inputs];
            layer.backward(
                &mut self.params,
                &pass.inputs[index],
                &gradient,
                Some(&mut below),
            );
            if index > 0 {
                for (delta, sum) in below.iter_mut().zip(&pass.sums[index - 1]) {
                    *delta *= if *sum > 0.0 { 1.0 } else { 0.0 };
                }
            }
            gradient = below;
        }
        loss
    }

    fn pass(&self, input: &[f32]) -> Pass {
        let mut pass = Pass {
            inputs: Vec::with_capacity(self.layers.len()),
            sums: Vec::with_capacity(self.layers.len()),
        };
        let mut current = input.to_vec();
        for (index, layer) in self.layers.iter().enumerate() {
            let mut sum = vec![0.0; layer.outputs];
            layer.forward(&self.params, &current, &mut sum);
            let hidden = index + 1 < self.layers.len();
            let next = if hidden {
                sum.iter().map(|value| value.max(0.0)).collect()
            } else {
                sum.clone()
            };
            pass.inputs.push(std::mem::replace(&mut current, next));
            pass.sums.push(sum);
        }
        pass
    }
}

fn cross_entropy(logit: f32, wanted: f32) -> f32 {
    logit.max(0.0) - logit * wanted + (1.0 + (-logit.abs()).exp()).ln()
}
