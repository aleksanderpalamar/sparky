use std::sync::Arc;

use super::{message::Message, panel_state::AgentClient, tray_test_support::bar, update::update};
use crate::domain::{
    agents::{Agent, AgentId, Snapshot},
    panel::PanelKind,
};
use crate::ports::agents::AgentControl;

#[derive(Debug)]
struct Fake;

impl AgentControl for Fake {
    fn snapshot(&self) -> Result<Snapshot, String> {
        Ok(Snapshot::default())
    }

    fn launch(&self, _: &AgentId) -> Result<(), String> {
        Ok(())
    }

    fn stop(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
}

fn setup() -> super::state::Bar {
    let mut bar = bar();
    bar.panels.agent_client = Some(AgentClient(Arc::new(Fake)));
    bar
}

#[test]
fn painel_abre_com_coleta_e_selector_fecha_ao_sair() {
    let mut bar = setup();
    let _ = update(&mut bar, Message::Panel(PanelKind::Agents));
    assert!(bar.panels.agent_loading);
    let id = bar.panels.agent_sequence;
    let snapshot = Snapshot {
        agents: vec![Agent {
            id: AgentId::Claude,
            name: "Claude Code".into(),
            tokens_today: Some(10),
            running: false,
            quotas: Vec::new(),
        }],
        local_error: None,
    };
    let _ = update(&mut bar, Message::AgentsSnapshot(id, Ok(snapshot.clone())));
    assert_eq!(bar.panels.agents, snapshot);
    assert!(!bar.panels.agent_loading);
    let _ = update(&mut bar, Message::AgentsSelector);
    assert!(bar.panels.agent_selector);
    let _ = update(&mut bar, Message::Panel(PanelKind::Agents));
    assert!(bar.panels.open.is_none());
    assert!(!bar.panels.agent_selector);
}

#[test]
fn resposta_antiga_nao_substitui_snapshot_atual() {
    let mut bar = setup();
    let _ = update(&mut bar, Message::Panel(PanelKind::Agents));
    let id = bar.panels.agent_sequence;
    let _ = update(
        &mut bar,
        Message::AgentsSnapshot(id + 1, Ok(Snapshot::default())),
    );
    assert!(bar.panels.agent_loading);
    let _ = update(
        &mut bar,
        Message::AgentsSnapshot(id, Ok(Snapshot::default())),
    );
    assert!(!bar.panels.agent_loading);
}

#[test]
fn abrir_agente_libera_teclado_do_painel() {
    let mut bar = setup();
    let _ = update(&mut bar, Message::Panel(PanelKind::Agents));
    let _ = update(&mut bar, Message::AgentsLaunch(AgentId::Codex));
    assert!(bar.panels.open.is_none());
}
