const SETTLE: f32 = 0.002;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ease {
    pub value: f32,
    pub target: f32,
}

impl Ease {
    pub fn at(value: f32) -> Self {
        Self {
            value,
            target: value,
        }
    }

    pub fn aim(&mut self, target: f32) {
        self.target = target;
    }

    pub fn step(&mut self, elapsed: f32, pace: f32) {
        let share = 1.0 - (-elapsed.max(0.0) / pace.max(f32::EPSILON)).exp();
        self.value += (self.target - self.value) * share;
        if (self.target - self.value).abs() < SETTLE {
            self.value = self.target;
        }
    }

    pub fn settled(&self) -> bool {
        self.value == self.target
    }
}
