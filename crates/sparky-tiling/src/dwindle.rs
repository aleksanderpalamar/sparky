use crate::geometry::{Arrangement, Rect};
use crate::ids::WindowId;
use crate::node::Node;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dwindle {
    root: Option<Node>,
}

impl Dwindle {
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn contains(&self, window: WindowId) -> bool {
        self.root.as_ref().is_some_and(|root| root.contains(window))
    }

    pub fn windows(&self) -> Vec<WindowId> {
        let mut found = Vec::new();
        if let Some(root) = &self.root {
            root.collect(&mut found);
        }
        found
    }

    pub fn insert(&mut self, window: WindowId, target: Option<WindowId>, area: Rect) {
        if self.contains(window) {
            return;
        }
        let Some(root) = self.root.take() else {
            self.root = Some(Node::Leaf(window));
            return;
        };
        let target = target
            .filter(|target| root.contains(*target))
            .unwrap_or_else(|| root.last_leaf());
        self.root = Some(root.insert(window, target, area));
    }

    pub fn remove(&mut self, window: WindowId) -> bool {
        let Some(root) = self.root.take() else {
            return false;
        };
        let (rest, removed) = root.remove(window);
        self.root = rest;
        removed
    }

    pub fn layout(&self, area: Rect) -> Vec<(WindowId, Rect)> {
        let mut placements = Vec::new();
        if let Some(root) = &self.root {
            root.layout(area, &mut placements);
        }
        placements
    }

    pub fn toggle_split(&mut self, window: WindowId) -> bool {
        let Some(split) = self.root.as_mut().and_then(|root| root.parent_of(window)) else {
            return false;
        };
        split.arrangement = match split.arrangement {
            Arrangement::SideBySide => Arrangement::Stacked,
            Arrangement::Stacked => Arrangement::SideBySide,
        };
        true
    }

    pub fn swap(&mut self, first: WindowId, second: WindowId) -> bool {
        if !self.contains(first) || !self.contains(second) {
            return false;
        }
        if let Some(root) = self.root.as_mut() {
            root.swap(first, second);
        }
        true
    }

    pub fn resize(&mut self, window: WindowId, delta_x: i32, delta_y: i32, area: Rect) {
        let Some(root) = self.root.as_mut() else {
            return;
        };
        if delta_x != 0 {
            root.resize(window, delta_x, Arrangement::SideBySide, area);
        }
        if delta_y != 0 {
            root.resize(window, delta_y, Arrangement::Stacked, area);
        }
    }
}
