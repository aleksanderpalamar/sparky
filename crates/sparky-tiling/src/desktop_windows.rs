use crate::desktop::{Desktop, Entry};
use crate::geometry::{Direction, Rect};
use crate::ids::{WindowId, WorkspaceId};
use crate::neighbor::neighbor;
use crate::workspace::{Fullscreen, Screen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opening {
    Tiled,
    Floating(i32, i32),
}

impl Desktop {
    pub fn add_window(&mut self, window: WindowId, opening: Opening) {
        let Some((id, name, screen)) = self.focused_target() else {
            return;
        };
        let workspace = &mut self.entry_mut(id, &name).workspace;
        match opening {
            Opening::Tiled => workspace.add_tiled(window, screen.area),
            Opening::Floating(width, height) => workspace.add_floating(window, screen.area.centered(width, height)),
        }
    }

    pub fn remove_window(&mut self, window: WindowId) -> bool {
        let removed = self.entries.values_mut().any(|entry| entry.workspace.remove(window));
        self.forget_empty();
        removed
    }

    pub fn workspace_of(&self, window: WindowId) -> Option<WorkspaceId> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.workspace.contains(window))
            .map(|(id, _)| *id)
    }

    pub fn focused_window(&self) -> Option<WindowId> {
        let id = self.active_workspace()?;
        self.entries.get(&id)?.workspace.focused()
    }

    pub fn focus_window(&mut self, window: WindowId) {
        let Some(id) = self.workspace_of(window) else {
            return;
        };
        self.focus_workspace(id);
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.workspace.focus(window);
        }
    }

    pub fn focus_direction(&mut self, direction: Direction) -> Option<WindowId> {
        if let Some(found) = self.neighbor_of_focused(direction) {
            self.focus_window(found);
            return Some(found);
        }
        let name = self.monitor_towards(direction)?;
        self.focus_monitor(&name);
        self.focused_window()
    }

    pub fn swap_direction(&mut self, direction: Direction) {
        let (Some(focused), Some(other)) = (self.focused_window(), self.neighbor_of_focused(direction)) else {
            return;
        };
        if let Some(entry) = self.active_entry_mut() {
            entry.workspace.swap(focused, other);
        }
    }

    pub fn swap_windows(&mut self, first: WindowId, second: WindowId) -> bool {
        let shared = self.workspace_of(first).filter(|id| first != second && self.workspace_of(second) == Some(*id));
        let Some(id) = shared else {
            return false;
        };
        self.entries.get_mut(&id).is_some_and(|entry| entry.workspace.swap(first, second))
    }

    pub fn toggle_floating(&mut self) {
        self.with_focused(|entry, window, screen, size| entry.workspace.toggle_floating(window, screen.area, size));
    }

    pub fn toggle_fullscreen(&mut self, mode: Fullscreen) {
        self.with_focused(|entry, window, _, _| entry.workspace.toggle_fullscreen(window, mode));
    }

    pub fn toggle_split(&mut self) {
        self.with_focused(|entry, window, _, _| {
            entry.workspace.toggle_split(window);
        });
    }

    pub fn resize_focused(&mut self, delta: (i32, i32)) {
        self.with_focused(|entry, window, screen, _| entry.workspace.resize(window, delta, screen.area));
    }

    pub fn set_floating_frame(&mut self, window: WindowId, frame: Rect) {
        for entry in self.entries.values_mut() {
            entry.workspace.set_floating_frame(window, frame);
        }
    }

    pub fn move_to_workspace(&mut self, window: WindowId, target: WorkspaceId, follow: bool) {
        let (Some(source), Some((_, here, fallback))) = (self.workspace_of(window), self.focused_target()) else {
            return;
        };
        if source == target {
            return;
        }
        let floating = self.entries.get(&source).is_some_and(|entry| entry.workspace.is_floating(window));
        if let Some(entry) = self.entries.get_mut(&source) {
            entry.workspace.remove(window);
        }
        let home = self.entry_mut(target, &here).monitor.clone();
        let screen = self.screen_named(&home).unwrap_or(fallback);
        let (width, height) = self.style.float_size;
        let destination = &mut self.entry_mut(target, &home).workspace;
        match floating {
            true => destination.add_floating(window, screen.area.centered(width, height)),
            false => destination.add_tiled(window, screen.area),
        }
        match follow {
            true => self.focus_window(window),
            false => self.forget_empty(),
        }
    }

    fn screen_named(&self, name: &str) -> Option<Screen> {
        self.monitors.iter().find(|monitor| monitor.name == name).map(|monitor| monitor.screen)
    }

    fn focused_target(&self) -> Option<(WorkspaceId, String, Screen)> {
        let monitor = self.focused_monitor()?;
        Some((monitor.active, monitor.name.clone(), monitor.screen))
    }

    fn active_entry_mut(&mut self) -> Option<&mut Entry> {
        let id = self.active_workspace()?;
        self.entries.get_mut(&id)
    }

    fn with_focused(&mut self, change: impl FnOnce(&mut Entry, WindowId, Screen, (i32, i32))) {
        let (Some(window), Some((_, _, screen))) = (self.focused_window(), self.focused_target()) else {
            return;
        };
        let size = self.style.float_size;
        if let Some(entry) = self.active_entry_mut() {
            change(entry, window, screen, size);
        }
    }

    fn neighbor_of_focused(&self, direction: Direction) -> Option<WindowId> {
        let focused = self.focused_window()?;
        let monitor = self.focused_monitor()?;
        let entry = self.entries.get(&monitor.active)?;
        let boxes: Vec<(WindowId, Rect)> = entry
            .workspace
            .arrange(monitor.screen, self.style.gaps)
            .into_iter()
            .map(|placement| (placement.window, placement.frame))
            .collect();
        neighbor(&boxes, focused, direction)
    }
}
