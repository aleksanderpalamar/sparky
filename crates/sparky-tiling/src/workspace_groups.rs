use crate::gaps::Gaps;
use crate::geometry::Rect;
use crate::groups::{BAR_HEIGHT, Group, Leaving};
use crate::ids::WindowId;
use crate::workspace::{Screen, Workspace};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs {
    pub frame: Rect,
    pub members: Vec<WindowId>,
    pub active: usize,
    pub focused: bool,
}

impl Workspace {
    pub fn toggle_group(&mut self, window: WindowId, area: Rect) {
        if let Some(leaf) = self.groups.of(window).map(Group::active) {
            for hidden in self.groups.dissolve(window) {
                self.tiled.insert(hidden, Some(leaf), area);
            }
            return;
        }
        if self.tiled.contains(window) {
            self.groups.create(window);
        }
    }

    pub fn leave_group(&mut self, window: WindowId, area: Rect) {
        let leaf = self.groups.of(window).map(Group::active);
        match self.groups.leave(window) {
            Leaving::Replaced(next) => {
                self.tiled.replace(window, next);
                self.tiled.insert(window, Some(next), area);
            }
            Leaving::Hidden => self.tiled.insert(window, leaf, area),
            Leaving::Last | Leaving::NotGrouped => {}
        }
        self.focus.touch(window);
    }

    pub fn join_group(&mut self, window: WindowId, target: WindowId) {
        let Some(group) = self.groups.of(target) else {
            return;
        };
        let leaf = group.active();
        if group.members().contains(&window) || !self.detach(window) {
            return;
        }
        self.join(leaf, window);
    }

    pub fn cycle_group(&mut self, window: WindowId, step: i32) {
        if let Some((old, new)) = self.groups.step(window, step) {
            self.reveal(old, new);
        }
    }

    pub fn select_in_group(&mut self, window: WindowId, index: usize) {
        if let Some((old, new)) = self.groups.select(window, index) {
            self.reveal(old, new);
        }
    }

    pub fn tabs(&self, screen: Screen, gaps: Gaps) -> Vec<Tabs> {
        let focused = self.focused();
        self.tiled
            .layout(screen.area)
            .into_iter()
            .filter(|(window, _)| self.fullscreen.is_none_or(|(full, _)| full != *window))
            .filter_map(|(window, tile)| {
                let group = self.groups.of(window)?;
                let frame = gaps.apply(tile, screen.area);
                Some(Tabs {
                    frame: Rect::new(frame.x, frame.y, frame.width, BAR_HEIGHT),
                    members: group.members().to_vec(),
                    active: group.active_index(),
                    focused: focused == Some(window),
                })
            })
            .collect()
    }

    pub(crate) fn framed(&self, window: WindowId, frame: Rect) -> Rect {
        match self.groups.of(window) {
            Some(_) => Rect::new(frame.x, frame.y + BAR_HEIGHT, frame.width, (frame.height - BAR_HEIGHT).max(1)),
            None => frame,
        }
    }

    pub(crate) fn hidden_members(&self) -> Vec<WindowId> {
        self.groups
            .all()
            .iter()
            .flat_map(|group| group.members().iter().copied().filter(|member| *member != group.active()))
            .collect()
    }

    pub(crate) fn show_member(&mut self, window: WindowId) {
        let index = self.groups.of(window).and_then(|group| group.members().iter().position(|member| *member == window));
        if let Some(index) = index.filter(|_| self.groups.is_hidden(window)) {
            self.select_in_group(window, index);
        }
    }

    pub(crate) fn detach(&mut self, window: WindowId) -> bool {
        match self.groups.leave(window) {
            Leaving::Replaced(next) => self.tiled.replace(window, next),
            Leaving::Hidden => true,
            Leaving::Last | Leaving::NotGrouped => self.tiled.remove(window),
        }
    }

    pub(crate) fn join(&mut self, leaf: WindowId, window: WindowId) {
        if self.groups.join(leaf, window) {
            self.tiled.replace(leaf, window);
            self.focus.touch(window);
        }
    }

    fn reveal(&mut self, old: WindowId, new: WindowId) {
        self.tiled.replace(old, new);
        self.focus.touch(new);
    }
}
