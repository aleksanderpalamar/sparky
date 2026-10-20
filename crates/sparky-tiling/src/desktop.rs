use std::collections::BTreeMap;

use crate::gaps::Gaps;
use crate::geometry::{Direction, Rect};
use crate::ids::WorkspaceId;
use crate::neighbor::neighbor;
use crate::workspace::{Placement, Screen, Workspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub gaps: Gaps,
    pub float_size: (i32, i32),
}

impl Style {
    pub const SPARKY: Self = Self {
        gaps: Gaps::SPARKY,
        float_size: (875, 600),
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    pub name: String,
    pub screen: Screen,
    pub active: WorkspaceId,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Entry {
    pub(crate) monitor: String,
    pub(crate) workspace: Workspace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Desktop {
    pub(crate) monitors: Vec<Monitor>,
    pub(crate) focused: usize,
    pub(crate) entries: BTreeMap<WorkspaceId, Entry>,
    pub(crate) previous: Option<WorkspaceId>,
    pub(crate) style: Style,
}

impl Desktop {
    pub fn new(style: Style) -> Self {
        Self {
            monitors: Vec::new(),
            focused: 0,
            entries: BTreeMap::new(),
            previous: None,
            style,
        }
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn monitors(&self) -> &[Monitor] {
        &self.monitors
    }

    pub fn focused_monitor(&self) -> Option<&Monitor> {
        self.monitors.get(self.focused)
    }

    pub fn add_monitor(&mut self, name: &str, full: Rect) {
        if self.monitors.iter().any(|monitor| monitor.name == name) {
            return;
        }
        let active = self.free_workspace();
        self.monitors.push(Monitor {
            name: name.to_owned(),
            screen: Screen { full, area: full },
            active,
        });
        self.adopt_orphans(name);
        self.entry_mut(active, name);
    }

    pub fn remove_monitor(&mut self, name: &str) {
        self.monitors.retain(|monitor| monitor.name != name);
        self.focused = self.focused.min(self.monitors.len().saturating_sub(1));
        if let Some(heir) = self.monitors.first().map(|monitor| monitor.name.clone()) {
            self.adopt_orphans(&heir);
        }
    }

    pub fn set_screen(&mut self, name: &str, screen: Screen) {
        if let Some(monitor) = self.monitors.iter_mut().find(|monitor| monitor.name == name) {
            monitor.screen = screen;
        }
    }

    pub fn focus_monitor(&mut self, name: &str) {
        if let Some(index) = self.monitors.iter().position(|monitor| monitor.name == name) {
            self.focused = index;
        }
    }

    pub fn focus_monitor_at(&mut self, x: i32, y: i32) {
        if let Some(index) = self.monitors.iter().position(|monitor| monitor.screen.full.contains(x, y)) {
            self.focused = index;
        }
    }

    pub fn cycle_monitor(&mut self, step: i32) {
        let count = self.monitors.len() as i32;
        if count > 0 {
            self.focused = (self.focused as i32 + step).rem_euclid(count) as usize;
        }
    }

    pub fn monitor_towards(&self, direction: Direction) -> Option<String> {
        let boxes: Vec<(usize, Rect)> = self
            .monitors
            .iter()
            .enumerate()
            .map(|(index, monitor)| (index, monitor.screen.full))
            .collect();
        let index = neighbor(&boxes, self.focused, direction)?;
        self.monitors.get(index).map(|monitor| monitor.name.clone())
    }

    pub fn arrange(&self) -> Vec<(String, Vec<Placement>)> {
        self.monitors
            .iter()
            .map(|monitor| {
                let placements = self
                    .entries
                    .get(&monitor.active)
                    .map(|entry| entry.workspace.arrange(monitor.screen, self.style.gaps))
                    .unwrap_or_default();
                (monitor.name.clone(), placements)
            })
            .collect()
    }

    pub(crate) fn entry_mut(&mut self, id: WorkspaceId, monitor: &str) -> &mut Entry {
        self.entries.entry(id).or_insert_with(|| Entry {
            monitor: monitor.to_owned(),
            workspace: Workspace::default(),
        })
    }

    fn free_workspace(&self) -> WorkspaceId {
        let taken = |id: u32| self.monitors.iter().any(|monitor| monitor.active.0 == id);
        let foreign = |id: u32| {
            self.entries
                .get(&WorkspaceId(id))
                .is_some_and(|entry| self.monitors.iter().any(|monitor| monitor.name == entry.monitor))
        };
        (WorkspaceId::FIRST.0..=WorkspaceId::LAST.0)
            .find(|id| !taken(*id) && !foreign(*id))
            .map(WorkspaceId)
            .unwrap_or(WorkspaceId::LAST)
    }

    fn adopt_orphans(&mut self, heir: &str) {
        let names: Vec<String> = self.monitors.iter().map(|monitor| monitor.name.clone()).collect();
        for entry in self.entries.values_mut() {
            if !names.contains(&entry.monitor) {
                entry.monitor = heir.to_owned();
            }
        }
    }
}
