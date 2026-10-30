use std::collections::VecDeque;

pub const SAMPLES: usize = 60;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    values: VecDeque<f32>,
}

impl History {
    pub fn push(&mut self, value: f32) {
        if self.values.len() == SAMPLES {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub fn values(&self) -> impl Iterator<Item = f32> + '_ {
        self.values.iter().copied()
    }

    pub fn latest(&self) -> Option<f32> {
        self.values.back().copied()
    }

    pub fn peak(&self) -> f32 {
        self.values.iter().copied().fold(0.0, f32::max)
    }
}
