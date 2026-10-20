use crate::desktop::Desktop;
use crate::ids::WorkspaceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub id: WorkspaceId,
    pub monitor: String,
    pub windows: usize,
    pub visible: bool,
    pub focused: bool,
}

impl Desktop {
    pub fn active_workspace(&self) -> Option<WorkspaceId> {
        self.focused_monitor().map(|monitor| monitor.active)
    }

    pub fn focus_workspace(&mut self, id: WorkspaceId) {
        let Some(monitor) = self.focused_monitor() else {
            return;
        };
        let (current, here) = (monitor.active, monitor.name.clone());
        if current != id {
            self.previous = Some(current);
        }
        let home = self.entries.get(&id).map_or(here, |entry| entry.monitor.clone());
        self.entry_mut(id, &home);
        let found = self.monitors.iter_mut().enumerate().find(|(_, monitor)| monitor.name == home);
        if let Some((index, monitor)) = found {
            monitor.active = id;
            self.focused = index;
        }
        self.forget_empty();
    }

    pub fn focus_relative(&mut self, step: i32) {
        let Some(current) = self.active_workspace() else {
            return;
        };
        let open: Vec<WorkspaceId> = self.entries.keys().copied().filter(WorkspaceId::is_numbered).collect();
        let Some(position) = open.iter().position(|id| *id == current) else {
            return;
        };
        let next = (position as i32 + step).rem_euclid(open.len() as i32) as usize;
        if let Some(target) = open.get(next).copied() {
            self.focus_workspace(target);
        }
    }

    pub fn focus_previous(&mut self) {
        if let Some(previous) = self.previous {
            self.focus_workspace(previous);
        }
    }

    pub fn summaries(&self) -> Vec<WorkspaceSummary> {
        let focused = self.active_workspace();
        self.entries
            .iter()
            .map(|(id, entry)| WorkspaceSummary {
                id: *id,
                monitor: entry.monitor.clone(),
                windows: entry.workspace.windows().len(),
                visible: self.monitors.iter().any(|monitor| monitor.active == *id),
                focused: focused == Some(*id),
            })
            .collect()
    }

    pub(crate) fn forget_empty(&mut self) {
        let visible: Vec<WorkspaceId> = self.monitors.iter().map(|monitor| monitor.active).collect();
        self.entries
            .retain(|id, entry| visible.contains(id) || !entry.workspace.is_empty());
    }
}
