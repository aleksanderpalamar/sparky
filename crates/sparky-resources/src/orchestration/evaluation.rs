use std::time::Duration;

pub const SETTLE: Duration = Duration::from_secs(30);
const IMPROVEMENT: f32 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    Waiting,
    Keep,
    Revert,
}

pub fn judge(before: f32, now: Option<f32>, elapsed: Duration) -> Judgement {
    if elapsed < SETTLE {
        return Judgement::Waiting;
    }
    match now {
        Some(now) if now <= before * IMPROVEMENT => Judgement::Keep,
        _ => Judgement::Revert,
    }
}
