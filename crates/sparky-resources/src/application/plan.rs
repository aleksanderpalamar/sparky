use super::encoding::{CONTEXT_INPUTS, OUTPUTS, STEP_INPUTS};
use crate::neural::Shape;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    pub capacity: usize,
    pub minimum: usize,
    pub epochs: usize,
    pub batch: usize,
    pub rate: f32,
    pub decay: f32,
    pub margin: f64,
    pub hidden: usize,
    pub blend: usize,
    pub seed: u64,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            capacity: 7200,
            minimum: 600,
            epochs: 20,
            batch: 16,
            rate: 0.001,
            decay: 0.05,
            margin: 0.05,
            hidden: 32,
            blend: 32,
            seed: 20261118,
        }
    }
}

pub fn shape(plan: &Plan) -> Shape {
    Shape {
        steps: STEP_INPUTS,
        context: CONTEXT_INPUTS,
        memory: plan.hidden,
        hidden: plan.blend,
        outputs: OUTPUTS,
    }
}
