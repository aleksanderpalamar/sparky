use crate::drag::{Edges, moved, resized};
use crate::geometry::Rect;
use crate::ids::WindowId;
use crate::workspace::Layer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Move,
    Resize(Edges),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Frame(Rect),
    Grow(i32, i32),
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Release {
    SwapWithUnder,
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Floating,
    Tiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drag {
    window: WindowId,
    gesture: Gesture,
    placement: Placement,
    frame: Rect,
    origin: (i32, i32),
    applied: (i32, i32),
}

impl Drag {
    pub fn start(window: WindowId, gesture: Gesture, layer: Layer, frame: Rect, origin: (i32, i32)) -> Option<Self> {
        let placement = match layer {
            Layer::Floating => Placement::Floating,
            Layer::Tiled => Placement::Tiled,
            Layer::Fullscreen(_) => return None,
        };
        Some(Self {
            window,
            gesture,
            placement,
            frame,
            origin,
            applied: (0, 0),
        })
    }

    pub fn window(&self) -> WindowId {
        self.window
    }

    pub fn follow(&mut self, point: (i32, i32)) -> Change {
        let total = (point.0 - self.origin.0, point.1 - self.origin.1);
        match (self.gesture, self.placement) {
            (Gesture::Move, Placement::Floating) => Change::Frame(moved(self.frame, total)),
            (Gesture::Resize(edges), Placement::Floating) => Change::Frame(resized(self.frame, edges, total)),
            (Gesture::Resize(edges), Placement::Tiled) => self.grow(edges.growth(total)),
            (Gesture::Move, Placement::Tiled) => Change::Nothing,
        }
    }

    pub fn release(&self) -> Release {
        match (self.gesture, self.placement) {
            (Gesture::Move, Placement::Tiled) => Release::SwapWithUnder,
            _ => Release::Nothing,
        }
    }

    fn grow(&mut self, wanted: (i32, i32)) -> Change {
        let step = (wanted.0 - self.applied.0, wanted.1 - self.applied.1);
        self.applied = wanted;
        Change::Grow(step.0, step.1)
    }
}
