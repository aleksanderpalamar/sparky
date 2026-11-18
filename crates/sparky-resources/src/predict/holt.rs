#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Holt {
    alpha: f32,
    beta: f32,
    state: Option<State>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct State {
    level: f32,
    trend: f32,
    at: f64,
}

impl Holt {
    pub fn new(alpha: f32, beta: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            beta: beta.clamp(0.0, 1.0),
            state: None,
        }
    }

    pub fn update(&mut self, at: f64, value: f32) {
        let Some(state) = self.state.filter(|state| at > state.at) else {
            self.state = self.state.or(Some(State {
                level: value,
                trend: 0.0,
                at,
            }));
            return;
        };
        let elapsed = (at - state.at) as f32;
        let projected = state.level + state.trend * elapsed;
        let level = self.alpha * value + (1.0 - self.alpha) * projected;
        let slope = (level - state.level) / elapsed;
        let trend = self.beta * slope + (1.0 - self.beta) * state.trend;
        self.state = Some(State { level, trend, at });
    }

    pub fn forecast(&self, ahead: f64) -> Option<f32> {
        let state = self.state?;
        Some(state.level + state.trend * ahead as f32)
    }
}
