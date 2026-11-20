use super::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub offset: usize,
    pub len: usize,
}

impl Slot {
    pub fn range(self) -> std::ops::Range<usize> {
        self.offset..self.offset + self.len
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parameters {
    pub values: Vec<f32>,
    pub grads: Vec<f32>,
}

impl Parameters {
    pub fn allocate(&mut self, len: usize) -> Slot {
        let slot = Slot {
            offset: self.values.len(),
            len,
        };
        self.values.resize(slot.offset + len, 0.0);
        self.grads.resize(slot.offset + len, 0.0);
        slot
    }

    pub fn randomize(&mut self, slot: Slot, fan_in: usize, fan_out: usize, rng: &mut Rng) {
        let limit = (6.0 / (fan_in + fan_out).max(1) as f32).sqrt();
        for value in &mut self.values[slot.range()] {
            *value = rng.uniform(limit);
        }
    }

    pub fn zero_grads(&mut self) {
        self.grads.fill(0.0);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn is_finite(&self) -> bool {
        self.values.iter().all(|value| value.is_finite())
    }
}
