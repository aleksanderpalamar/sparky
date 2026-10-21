use crate::desktop::{Desktop, Monitor};
use crate::ids::{WindowId, WorkspaceId};
use crate::workspace::Placement;

impl Desktop {
    pub fn toggle_scratchpad(&mut self) {
        let Some(here) = self.focused_monitor().map(|monitor| monitor.name.clone()) else {
            return;
        };
        match self.scratchpad.as_deref() == Some(here.as_str()) {
            true => self.scratchpad = None,
            false => self.show_scratchpad(),
        }
    }

    pub fn move_to_scratchpad(&mut self, window: WindowId) {
        self.move_to_workspace(window, WorkspaceId::SCRATCHPAD, false);
    }

    pub fn scratchpad_shown(&self) -> Option<&str> {
        self.scratchpad.as_deref()
    }

    pub fn is_covered(&self, window: WindowId) -> bool {
        let Some(id) = self.workspace_of(window) else {
            return false;
        };
        self.monitors
            .iter()
            .any(|monitor| monitor.active == id && self.scratchpad.as_deref() == Some(monitor.name.as_str()))
    }

    pub(crate) fn show_scratchpad(&mut self) {
        let Some(monitor) = self.focused_monitor() else {
            return;
        };
        let (here, area) = (monitor.name.clone(), monitor.screen.area);
        let before = self.entries.get(&WorkspaceId::SCRATCHPAD).map(|entry| entry.monitor.clone());
        let origin = before.and_then(|name| self.monitors.iter().find(|known| known.name == name)).map(|known| known.screen.area);
        let entry = self.entry_mut(WorkspaceId::SCRATCHPAD, &here);
        if let Some(origin) = origin {
            entry.workspace.translate_floating(area.x - origin.x, area.y - origin.y);
        }
        entry.monitor = here.clone();
        self.scratchpad = Some(here);
    }

    pub(crate) fn shown(&self, monitor: &Monitor) -> WorkspaceId {
        match self.scratchpad.as_deref() == Some(monitor.name.as_str()) {
            true => WorkspaceId::SCRATCHPAD,
            false => monitor.active,
        }
    }

    pub(crate) fn scratchpad_on(&self, monitor: &Monitor) -> Vec<Placement> {
        if self.scratchpad.as_deref() != Some(monitor.name.as_str()) {
            return Vec::new();
        }
        self.entries
            .get(&WorkspaceId::SCRATCHPAD)
            .map(|entry| entry.workspace.arrange(monitor.screen, self.style.gaps))
            .unwrap_or_default()
    }
}
