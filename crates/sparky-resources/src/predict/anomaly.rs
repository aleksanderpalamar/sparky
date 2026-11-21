const WARMUP: u32 = 30;
const FLOOR: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deviation {
    weight: f32,
    mean: f32,
    variance: f32,
    seen: u32,
}

impl Deviation {
    pub fn new(weight: f32) -> Self {
        Self {
            weight: weight.clamp(0.0, 1.0),
            mean: 0.0,
            variance: 0.0,
            seen: 0,
        }
    }

    pub fn score(&self, value: f32) -> Option<f32> {
        if self.seen < WARMUP {
            return None;
        }
        let spread = self.variance.sqrt().max(FLOOR);
        Some((value - self.mean).abs() / spread)
    }

    pub fn update(&mut self, value: f32) {
        if self.seen == 0 {
            self.mean = value;
        }
        let difference = value - self.mean;
        self.mean += self.weight * difference;
        self.variance =
            (1.0 - self.weight) * (self.variance + self.weight * difference * difference);
        self.seen = self.seen.saturating_add(1);
    }
}
