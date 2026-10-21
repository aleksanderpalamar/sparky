use crate::ids::WindowId;

pub const BAR_HEIGHT: i32 = 22;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    members: Vec<WindowId>,
    active: usize,
}

impl Group {
    pub fn members(&self) -> &[WindowId] {
        &self.members
    }

    pub fn active(&self) -> WindowId {
        self.members[self.active]
    }

    pub fn active_index(&self) -> usize {
        self.active
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaving {
    NotGrouped,
    Hidden,
    Replaced(WindowId),
    Last,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Groups {
    groups: Vec<Group>,
}

impl Groups {
    pub fn all(&self) -> &[Group] {
        &self.groups
    }

    pub fn of(&self, window: WindowId) -> Option<&Group> {
        self.groups.iter().find(|group| group.members.contains(&window))
    }

    pub fn is_hidden(&self, window: WindowId) -> bool {
        self.of(window).is_some_and(|group| group.active() != window)
    }

    pub fn create(&mut self, window: WindowId) {
        if self.of(window).is_none() {
            self.groups.push(Group { members: vec![window], active: 0 });
        }
    }

    pub fn join(&mut self, target: WindowId, window: WindowId) -> bool {
        if self.of(window).is_some() {
            return false;
        }
        let Some(group) = self.group_mut(target) else {
            return false;
        };
        group.active += 1;
        group.members.insert(group.active, window);
        true
    }

    pub fn leave(&mut self, window: WindowId) -> Leaving {
        let Some(index) = self.groups.iter().position(|group| group.members.contains(&window)) else {
            return Leaving::NotGrouped;
        };
        let group = &mut self.groups[index];
        let active = group.active();
        group.members.retain(|member| *member != window);
        if group.members.is_empty() {
            self.groups.remove(index);
            return Leaving::Last;
        }
        if active == window {
            group.active = group.active.min(group.members.len() - 1);
            return Leaving::Replaced(group.active());
        }
        group.active = group.members.iter().position(|member| *member == active).unwrap_or(0);
        Leaving::Hidden
    }

    pub fn dissolve(&mut self, window: WindowId) -> Vec<WindowId> {
        let Some(index) = self.groups.iter().position(|group| group.members.contains(&window)) else {
            return Vec::new();
        };
        let group = self.groups.remove(index);
        let active = group.active();
        group.members.into_iter().filter(|member| *member != active).collect()
    }

    pub fn step(&mut self, window: WindowId, step: i32) -> Option<(WindowId, WindowId)> {
        let group = self.group_mut(window)?;
        let count = group.members.len() as i32;
        let target = (group.active as i32 + step).rem_euclid(count) as usize;
        Self::switch(group, target)
    }

    pub fn select(&mut self, window: WindowId, index: usize) -> Option<(WindowId, WindowId)> {
        let group = self.group_mut(window)?;
        if index >= group.members.len() {
            return None;
        }
        Self::switch(group, index)
    }

    fn switch(group: &mut Group, index: usize) -> Option<(WindowId, WindowId)> {
        let old = group.active();
        group.active = index;
        Some((old, group.active()))
    }

    fn group_mut(&mut self, window: WindowId) -> Option<&mut Group> {
        self.groups.iter_mut().find(|group| group.members.contains(&window))
    }
}
