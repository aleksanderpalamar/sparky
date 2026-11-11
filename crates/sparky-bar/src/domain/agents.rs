#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentId {
    Claude,
    Codex,
    Local(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub tokens_today: Option<u64>,
    pub running: bool,
    pub quotas: Vec<Quota>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    FiveHours,
    Weekly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quota {
    pub period: Period,
    pub used_percent: u8,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub agents: Vec<Agent>,
    pub local_error: Option<String>,
}

impl Snapshot {
    pub fn tokens_today(&self) -> u64 {
        self.agents
            .iter()
            .filter_map(|agent| agent.tokens_today)
            .fold(0, u64::saturating_add)
    }
}

#[cfg(test)]
mod tests {
    use super::{Agent, AgentId, Snapshot};

    #[test]
    fn total_ignora_modelos_sem_contagem_e_nao_estoura() {
        let snapshot = Snapshot {
            agents: vec![
                Agent {
                    id: AgentId::Claude,
                    name: "Claude Code".into(),
                    tokens_today: Some(u64::MAX),
                    running: false,
                    quotas: Vec::new(),
                },
                Agent {
                    id: AgentId::Codex,
                    name: "Codex".into(),
                    tokens_today: Some(1),
                    running: false,
                    quotas: Vec::new(),
                },
                Agent {
                    id: AgentId::Local("qwen".into()),
                    name: "qwen".into(),
                    tokens_today: None,
                    running: true,
                    quotas: Vec::new(),
                },
            ],
            local_error: None,
        };
        assert_eq!(snapshot.tokens_today(), u64::MAX);
    }
}
