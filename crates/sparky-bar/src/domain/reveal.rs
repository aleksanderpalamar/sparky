use std::time::{Duration, Instant};

pub const DURATION: Duration = Duration::from_millis(600);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Expansion {
    #[default]
    Closed,
    Open,
}

impl Expansion {
    fn value(self) -> f32 {
        match self {
            Self::Closed => 0.0,
            Self::Open => 1.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Reveal {
    progress: f32,
    origin: f32,
    target: Expansion,
    started: Option<Instant>,
}

impl Reveal {
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn animating(&self) -> bool {
        self.started.is_some()
    }

    pub fn target(&self) -> Expansion {
        self.target
    }

    pub fn set(&mut self, target: Expansion, now: Instant) {
        if target == self.target {
            return;
        }
        self.advance(now);
        self.origin = self.progress;
        self.target = target;
        self.started = Some(now);
    }

    pub fn advance(&mut self, now: Instant) {
        let Some(started) = self.started else {
            return;
        };
        let fraction = (now.saturating_duration_since(started).as_secs_f32()
            / DURATION.as_secs_f32())
        .min(1.0);
        let eased = 1.0 - (1.0 - fraction).powi(3);
        self.progress = self.origin + (self.target.value() - self.origin) * eased;
        if fraction == 1.0 {
            self.started = None;
        }
    }
}
