use crate::domain::agents::{AgentId, Snapshot};

pub trait AgentControl: Send + Sync + std::fmt::Debug {
    fn snapshot(&self) -> Result<Snapshot, String>;
    fn launch(&self, agent: &AgentId) -> Result<(), String>;
    fn stop(&self, model: &str) -> Result<(), String>;
}
