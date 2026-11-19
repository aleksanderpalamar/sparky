use sparky_resources::domain::{Change, IoClass};
use sparky_resources::orchestration::{Decision, Verdict};
use sparky_resources::safety::Outcome;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lens {
    #[default]
    Overview,
    Decisions,
    Learning,
}

impl Lens {
    pub const ALL: [Lens; 3] = [Lens::Overview, Lens::Decisions, Lens::Learning];

    pub fn key(self) -> &'static str {
        match self {
            Lens::Overview => "lens-overview",
            Lens::Decisions => "lens-decisions",
            Lens::Learning => "lens-learning",
        }
    }

    pub fn next(self) -> Lens {
        self.shifted(1)
    }

    pub fn previous(self) -> Lens {
        self.shifted(Self::ALL.len() - 1)
    }

    fn shifted(self, step: usize) -> Lens {
        let index = Self::ALL.iter().position(|lens| *lens == self).unwrap_or(0);
        Self::ALL[(index + step) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Positive,
    Neutral,
    Protective,
    Negative,
}

pub fn tone(verdict: &Verdict) -> Tone {
    match verdict {
        Verdict::Applied | Verdict::Kept => Tone::Positive,
        Verdict::Simulated | Verdict::Finished(Outcome::Restored | Outcome::Gone) => Tone::Neutral,
        Verdict::Blocked(_) => Tone::Protective,
        Verdict::Reverted(_) | Verdict::Failed | Verdict::Finished(_) => Tone::Negative,
    }
}

pub fn title(change: &Change) -> &'static str {
    match change {
        Change::Nice(_) => "change-nice",
        Change::Affinity(_) => "change-affinity",
        Change::IoPriority(_) => "change-io",
    }
}

pub fn detail(change: &Change) -> String {
    match change {
        Change::Nice(value) => format!("nice {value}"),
        Change::Affinity(cpus) => {
            let list: Vec<String> = cpus.iter().map(u32::to_string).collect();
            format!("CPU {}", list.join(", "))
        }
        Change::IoPriority(IoClass::None) => "io none".to_owned(),
        Change::IoPriority(IoClass::BestEffort(level)) => format!("io best-effort {level}"),
        Change::IoPriority(IoClass::Idle) => "io idle".to_owned(),
    }
}

pub fn regressions(decisions: &[Decision]) -> usize {
    decisions
        .iter()
        .filter(|decision| matches!(decision.verdict, Verdict::Reverted(_)))
        .count()
}
