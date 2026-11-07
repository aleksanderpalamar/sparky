use sparky_ipc::{OutputInfo, SessionInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    NoCompositor,
    Unsupported,
    TimedOut,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub info: Result<SessionInfo, Problem>,
    pub outputs: Result<Vec<OutputInfo>, Problem>,
}

impl Snapshot {
    pub fn absent(problem: Problem) -> Self {
        Self {
            info: Err(problem.clone()),
            outputs: Err(problem),
        }
    }
}

pub trait Compositor: Send + Sync {
    fn snapshot(&self) -> Snapshot;
}
