use std::time::{Duration, Instant};

use iced::Task;

use super::{message::Message, panel_actions, state::Bar};
use crate::domain::{
    agents::{AgentId, Snapshot},
    panel::PanelKind,
};

const INTERVAL: Duration = Duration::from_secs(300);

pub fn refresh_if_due(bar: &mut Bar) -> Task<Message> {
    if bar
        .panels
        .open
        .is_none_or(|open| open.kind != PanelKind::Agents)
    {
        return Task::none();
    }
    if bar
        .panels
        .agent_last_refresh
        .is_some_and(|last| last.elapsed() < INTERVAL)
    {
        return Task::none();
    }
    refresh(bar)
}

pub fn refresh(bar: &mut Bar) -> Task<Message> {
    let Some(client) = bar.panels.agent_client.clone() else {
        return Task::none();
    };
    if bar.panels.agent_loading {
        return Task::none();
    }
    bar.panels.agent_loading = true;
    bar.panels.agent_sequence = bar.panels.agent_sequence.wrapping_add(1);
    let id = bar.panels.agent_sequence;
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || client.0.snapshot())
                .await
                .map_err(|error| error.to_string())?
        },
        move |result| Message::AgentsSnapshot(id, result),
    )
}

pub fn received(bar: &mut Bar, id: u64, result: Result<Snapshot, String>) {
    if id != bar.panels.agent_sequence {
        return;
    }
    bar.panels.agent_loading = false;
    bar.panels.agent_last_refresh = Some(Instant::now());
    match result {
        Ok(snapshot) => {
            bar.panels.agents = snapshot;
            bar.panels.errors.remove(&PanelKind::Agents);
        }
        Err(error) => {
            bar.panels.errors.insert(PanelKind::Agents, error);
        }
    }
}

pub fn launch(bar: &mut Bar, agent: AgentId) -> Task<Message> {
    let Some(client) = bar.panels.agent_client.clone() else {
        return Task::none();
    };
    let close = panel_actions::close(bar);
    let launch = Task::perform(
        async move {
            tokio::task::spawn_blocking(move || client.0.launch(&agent))
                .await
                .map_err(|error| error.to_string())?
        },
        Message::AgentsLaunched,
    );
    Task::batch([close, launch])
}

pub fn launched(bar: &mut Bar, result: Result<(), String>) {
    if let Err(error) = result {
        bar.panels.errors.insert(PanelKind::Agents, error);
    }
}

pub fn stop(bar: &mut Bar, model: String) -> Task<Message> {
    let Some(client) = bar.panels.agent_client.clone() else {
        return Task::none();
    };
    if bar.panels.pending.contains_key(&PanelKind::Agents) {
        return Task::none();
    }
    bar.panels.begin(PanelKind::Agents);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || client.0.stop(&model))
                .await
                .map_err(|error| error.to_string())?
        },
        Message::AgentsStopped,
    )
}

pub fn stopped(bar: &mut Bar, result: Result<(), String>) -> Task<Message> {
    let Some(pending) = bar.panels.pending.get(&PanelKind::Agents).copied() else {
        return Task::none();
    };
    let success = result.is_ok() && pending.origin == bar.panels.open.map(|open| open.window);
    bar.panels.finish(pending.id, PanelKind::Agents, result);
    if success { refresh(bar) } else { Task::none() }
}
