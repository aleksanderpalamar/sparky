use std::time::Duration;

pub const MAX_FRAMES: usize = 8;
pub const MAX_SESSIONS: usize = 16;
pub const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Ready,
    Later(Duration),
    Unchanged,
}

#[derive(Debug, Default)]
pub struct Pace {
    last: Option<Duration>,
    generation: Option<u64>,
}

impl Pace {
    pub fn gate(&self, now: Duration, generation: u64) -> Gate {
        if self.generation == Some(generation) {
            return Gate::Unchanged;
        }
        let Some(last) = self.last else {
            return Gate::Ready;
        };
        let due = last.saturating_add(FRAME_INTERVAL);
        match due.checked_sub(now) {
            Some(wait) if !wait.is_zero() => Gate::Later(wait),
            _ => Gate::Ready,
        }
    }

    pub fn submitted(&mut self, now: Duration, generation: u64) {
        self.last = Some(now);
        self.generation = Some(generation);
    }
}

pub fn admits(pending: usize, inflight: usize) -> bool {
    pending
        .checked_add(inflight)
        .is_some_and(|count| count < MAX_FRAMES)
}

pub fn valid_damage(x: i32, y: i32, width: i32, height: i32) -> bool {
    x >= 0 && y >= 0 && width > 0 && height > 0
}

pub fn replaces_wake(current: Option<Duration>, requested: Duration) -> bool {
    current.is_none_or(|deadline| requested < deadline)
}

pub fn can_publish(submitted_locked: bool, current_locked: bool) -> bool {
    submitted_locked == current_locked
}
