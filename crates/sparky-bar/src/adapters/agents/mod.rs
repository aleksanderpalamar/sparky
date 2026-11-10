mod binary;
mod ollama;
mod quota_claude;
mod quota_codex;
mod usage;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::thread;

use crate::domain::agents::{Agent, AgentId, Snapshot};
use crate::ports::agents::AgentControl;

#[derive(Debug)]
pub struct SystemAgents {
    usage: Mutex<usage::Usage>,
    home: PathBuf,
}

impl Default for SystemAgents {
    fn default() -> Self {
        Self {
            usage: Mutex::new(usage::Usage::default()),
            home: env::var_os("HOME").map(PathBuf::from).unwrap_or_default(),
        }
    }
}

impl AgentControl for SystemAgents {
    fn snapshot(&self) -> Result<Snapshot, String> {
        let claude = binary::find("claude", &self.home);
        let codex = binary::find("codex", &self.home);
        let local = binary::find("ollama", &self.home);
        let mut snapshot = Snapshot::default();
        let claude_config = self.claude_config();
        let claude_quota = claude
            .as_ref()
            .map(|_| thread::spawn(move || quota_claude::read(&claude_config)));
        let codex_quota = codex.as_ref().map(|binary| {
            let binary = binary.clone();
            thread::spawn(move || quota_codex::read(&binary))
        });
        let (claude_tokens, codex_tokens) = if claude.is_some() || codex.is_some() {
            self.usage
                .lock()
                .map_err(|error| error.to_string())?
                .today(&self.claude_root(), &self.codex_root())
        } else {
            (0, 0)
        };
        if claude.is_some() {
            snapshot.agents.push(Agent {
                id: AgentId::Claude,
                name: "Claude Code".to_owned(),
                tokens_today: Some(claude_tokens),
                running: false,
                quotas: claude_quota
                    .and_then(|task| task.join().ok())
                    .unwrap_or_default(),
            });
        }
        if codex.is_some() {
            snapshot.agents.push(Agent {
                id: AgentId::Codex,
                name: "Codex".to_owned(),
                tokens_today: Some(codex_tokens),
                running: false,
                quotas: codex_quota
                    .and_then(|task| task.join().ok())
                    .unwrap_or_default(),
            });
        }
        if let Some(binary) = local {
            match ollama::models(&binary) {
                Ok(models) => snapshot
                    .agents
                    .extend(models.into_iter().map(|model| Agent {
                        id: AgentId::Local(model.name.clone()),
                        name: model.name,
                        tokens_today: None,
                        running: model.running,
                        quotas: Vec::new(),
                    })),
                Err(error) => snapshot.local_error = Some(error),
            }
        }
        Ok(snapshot)
    }

    fn launch(&self, agent: &AgentId) -> Result<(), String> {
        let (program, args) = match agent {
            AgentId::Claude => (binary::find("claude", &self.home), Vec::new()),
            AgentId::Codex => (binary::find("codex", &self.home), Vec::new()),
            AgentId::Local(name) => {
                let binary = binary::find("ollama", &self.home);
                let Some(path) = binary.as_ref() else {
                    return Err(crate::i18n::text("agents-not-installed"));
                };
                if !ollama::models(path)?
                    .iter()
                    .any(|model| &model.name == name)
                {
                    return Err(crate::i18n::text("agents-not-installed"));
                }
                (binary, vec!["run".to_owned(), name.clone()])
            }
        };
        let Some(program) = program else {
            return Err(crate::i18n::text("agents-not-installed"));
        };
        let Some(terminal) = binary::find("sparky-terminal", &self.home) else {
            return Err(crate::i18n::text("agents-terminal-missing"));
        };
        Command::new(terminal)
            .arg("-e")
            .arg(program)
            .args(args)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn stop(&self, model: &str) -> Result<(), String> {
        let Some(binary) = binary::find("ollama", &self.home) else {
            return Err(crate::i18n::text("agents-not-installed"));
        };
        if !ollama::models(&binary)?
            .iter()
            .any(|entry| entry.name == model && entry.running)
        {
            return Err(crate::i18n::text("agents-not-running"));
        }
        ollama::stop(&binary, model)
    }
}

impl SystemAgents {
    fn claude_root(&self) -> PathBuf {
        self.claude_config().join("projects")
    }

    fn claude_config(&self) -> PathBuf {
        env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home.join(".claude"))
    }

    fn codex_root(&self) -> PathBuf {
        env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home.join(".codex"))
            .join("sessions")
    }
}
