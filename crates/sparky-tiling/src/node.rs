use crate::geometry::{Arrangement, Rect};
use crate::ids::WindowId;

const DEFAULT_RATIO: f32 = 0.5;
const MIN_RATIO: f32 = 0.1;
const MAX_RATIO: f32 = 0.9;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Leaf(WindowId),
    Split(Box<Split>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub arrangement: Arrangement,
    pub ratio: f32,
    pub first: Node,
    pub second: Node,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Search {
    Missing,
    Unclaimed,
    Applied,
}

impl Node {
    pub fn contains(&self, window: WindowId) -> bool {
        match self {
            Node::Leaf(id) => *id == window,
            Node::Split(split) => split.first.contains(window) || split.second.contains(window),
        }
    }

    pub fn collect(&self, found: &mut Vec<WindowId>) {
        match self {
            Node::Leaf(id) => found.push(*id),
            Node::Split(split) => {
                split.first.collect(found);
                split.second.collect(found);
            }
        }
    }

    pub fn last_leaf(&self) -> WindowId {
        match self {
            Node::Leaf(id) => *id,
            Node::Split(split) => split.second.last_leaf(),
        }
    }

    pub fn insert(self, window: WindowId, target: WindowId, area: Rect) -> Node {
        match self {
            Node::Leaf(id) if id == target => Node::Split(Box::new(Split::new(
                area.arrangement(),
                Node::Leaf(id),
                Node::Leaf(window),
            ))),
            Node::Leaf(id) => Node::Leaf(id),
            Node::Split(split) => {
                let (first_area, second_area) = area.split(split.arrangement, split.ratio);
                let Split { arrangement, ratio, first, second } = *split;
                Node::Split(Box::new(Split {
                    arrangement,
                    ratio,
                    first: first.insert(window, target, first_area),
                    second: second.insert(window, target, second_area),
                }))
            }
        }
    }

    pub fn remove(self, window: WindowId) -> (Option<Node>, bool) {
        match self {
            Node::Leaf(id) if id == window => (None, true),
            Node::Leaf(id) => (Some(Node::Leaf(id)), false),
            Node::Split(split) => {
                let Split { arrangement, ratio, first, second } = *split;
                let (first, removed_first) = first.remove(window);
                let (second, removed_second) = second.remove(window);
                let rest = match (first, second) {
                    (Some(first), Some(second)) => Some(Node::Split(Box::new(Split {
                        arrangement,
                        ratio,
                        first,
                        second,
                    }))),
                    (Some(only), None) | (None, Some(only)) => Some(only),
                    (None, None) => None,
                };
                (rest, removed_first || removed_second)
            }
        }
    }

    pub fn layout(&self, area: Rect, placements: &mut Vec<(WindowId, Rect)>) {
        match self {
            Node::Leaf(id) => placements.push((*id, area)),
            Node::Split(split) => {
                let (first, second) = area.split(split.arrangement, split.ratio);
                split.first.layout(first, placements);
                split.second.layout(second, placements);
            }
        }
    }

    pub fn parent_of(&mut self, window: WindowId) -> Option<&mut Split> {
        let Node::Split(split) = self else {
            return None;
        };
        let direct = matches!(split.first, Node::Leaf(id) if id == window)
            || matches!(split.second, Node::Leaf(id) if id == window);
        if direct {
            return Some(split);
        }
        match split.first.contains(window) {
            true => split.first.parent_of(window),
            false => split.second.parent_of(window),
        }
    }

    pub fn swap(&mut self, first: WindowId, second: WindowId) {
        match self {
            Node::Leaf(id) if *id == first => *id = second,
            Node::Leaf(id) if *id == second => *id = first,
            Node::Leaf(_) => {}
            Node::Split(split) => {
                split.first.swap(first, second);
                split.second.swap(first, second);
            }
        }
    }

    pub fn resize(&mut self, window: WindowId, delta: i32, along: Arrangement, area: Rect) -> Search {
        match self {
            Node::Leaf(id) if *id == window => Search::Unclaimed,
            Node::Leaf(_) => Search::Missing,
            Node::Split(split) => split.resize(window, delta, along, area),
        }
    }
}

impl Split {
    pub fn new(arrangement: Arrangement, first: Node, second: Node) -> Self {
        Self {
            arrangement,
            ratio: DEFAULT_RATIO,
            first,
            second,
        }
    }

    fn resize(&mut self, window: WindowId, delta: i32, along: Arrangement, area: Rect) -> Search {
        let (first_area, second_area) = area.split(self.arrangement, self.ratio);
        let in_first = self.first.contains(window);
        let found = match in_first {
            true => self.first.resize(window, delta, along, first_area),
            false => self.second.resize(window, delta, along, second_area),
        };
        if found != Search::Unclaimed || self.arrangement != along {
            return found;
        }
        let length = match along {
            Arrangement::SideBySide => area.width,
            Arrangement::Stacked => area.height,
        };
        let change = delta as f32 / length.max(1) as f32;
        let signed = if in_first { change } else { -change };
        self.ratio = (self.ratio + signed).clamp(MIN_RATIO, MAX_RATIO);
        Search::Applied
    }
}
