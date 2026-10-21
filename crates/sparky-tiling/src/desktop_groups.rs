use crate::desktop::{Desktop, Monitor};
use crate::geometry::Direction;
use crate::ids::WorkspaceId;
use crate::workspace_groups::Tabs;

impl Desktop {
    pub fn toggle_group(&mut self) {
        self.with_focused(|entry, window, screen, _| entry.workspace.toggle_group(window, screen.area));
    }

    pub fn leave_group(&mut self) {
        self.with_focused(|entry, window, screen, _| entry.workspace.leave_group(window, screen.area));
    }

    pub fn move_into_group(&mut self, direction: Direction) {
        let Some(target) = self.neighbor_of_focused(direction) else {
            return;
        };
        self.with_focused(|entry, window, _, _| entry.workspace.join_group(window, target));
    }

    pub fn cycle_group(&mut self, step: i32) {
        self.with_focused(|entry, window, _, _| entry.workspace.cycle_group(window, step));
    }

    pub fn select_in_group(&mut self, index: usize) {
        self.with_focused(|entry, window, _, _| entry.workspace.select_in_group(window, index));
    }

    pub fn tabs(&self) -> Vec<(String, Vec<Tabs>)> {
        self.monitors
            .iter()
            .map(|monitor| {
                let mut tabs = self.tabs_of(monitor.active, monitor);
                if self.scratchpad_shown() == Some(monitor.name.as_str()) {
                    tabs.extend(self.tabs_of(WorkspaceId::SCRATCHPAD, monitor));
                }
                (monitor.name.clone(), tabs)
            })
            .collect()
    }

    fn tabs_of(&self, id: WorkspaceId, monitor: &Monitor) -> Vec<Tabs> {
        self.entries
            .get(&id)
            .map(|entry| entry.workspace.tabs(monitor.screen, self.style.gaps))
            .unwrap_or_default()
    }
}
