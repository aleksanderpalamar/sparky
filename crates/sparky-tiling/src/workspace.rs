use crate::dwindle::Dwindle;
use crate::focus::FocusHistory;
use crate::gaps::Gaps;
use crate::geometry::Rect;
use crate::ids::WindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fullscreen {
    Monitor,
    Maximized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Tiled,
    Floating,
    Fullscreen(Fullscreen),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub window: WindowId,
    pub frame: Rect,
    pub layer: Layer,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Screen {
    pub full: Rect,
    pub area: Rect,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Workspace {
    tiled: Dwindle,
    floating: Vec<(WindowId, Rect)>,
    fullscreen: Option<(WindowId, Fullscreen)>,
    focus: FocusHistory,
}

impl Workspace {
    pub fn is_empty(&self) -> bool {
        self.tiled.is_empty() && self.floating.is_empty()
    }

    pub fn contains(&self, window: WindowId) -> bool {
        self.tiled.contains(window) || self.is_floating(window)
    }

    pub fn is_floating(&self, window: WindowId) -> bool {
        self.floating.iter().any(|(known, _)| *known == window)
    }

    pub fn windows(&self) -> Vec<WindowId> {
        let floating = self.floating.iter().map(|(window, _)| *window);
        self.tiled.windows().into_iter().chain(floating).collect()
    }

    pub fn focused(&self) -> Option<WindowId> {
        self.focus.current()
    }

    pub fn focus(&mut self, window: WindowId) {
        if self.contains(window) {
            self.focus.touch(window);
        }
    }

    pub fn add_tiled(&mut self, window: WindowId, area: Rect) {
        let target = self.focused().filter(|focused| self.tiled.contains(*focused));
        self.tiled.insert(window, target, area);
        self.focus.touch(window);
    }

    pub fn add_floating(&mut self, window: WindowId, frame: Rect) {
        if !self.contains(window) {
            self.floating.push((window, frame));
        }
        self.focus.touch(window);
    }

    pub fn remove(&mut self, window: WindowId) -> bool {
        let tiled = self.tiled.remove(window);
        let before = self.floating.len();
        self.floating.retain(|(known, _)| *known != window);
        if self.fullscreen.is_some_and(|(known, _)| known == window) {
            self.fullscreen = None;
        }
        self.focus.forget(window);
        tiled || before != self.floating.len()
    }

    pub fn toggle_floating(&mut self, window: WindowId, area: Rect, size: (i32, i32)) {
        if self.tiled.remove(window) {
            self.floating.push((window, area.centered(size.0, size.1)));
            return;
        }
        if self.is_floating(window) {
            self.floating.retain(|(known, _)| *known != window);
            let target = self.tiled.windows().last().copied();
            self.tiled.insert(window, target, area);
        }
    }

    pub fn toggle_fullscreen(&mut self, window: WindowId, mode: Fullscreen) {
        if !self.contains(window) {
            return;
        }
        self.fullscreen = match self.fullscreen {
            Some(current) if current == (window, mode) => None,
            _ => Some((window, mode)),
        };
    }

    pub fn toggle_split(&mut self, window: WindowId) -> bool {
        self.tiled.toggle_split(window)
    }

    pub fn swap(&mut self, first: WindowId, second: WindowId) -> bool {
        self.tiled.swap(first, second)
    }

    pub fn resize(&mut self, window: WindowId, delta: (i32, i32), area: Rect) {
        if self.tiled.contains(window) {
            self.tiled.resize(window, delta.0, delta.1, area);
            return;
        }
        if let Some((_, frame)) = self.floating.iter_mut().find(|(known, _)| *known == window) {
            frame.width = (frame.width + delta.0).max(1);
            frame.height = (frame.height + delta.1).max(1);
        }
    }

    pub fn set_floating_frame(&mut self, window: WindowId, frame: Rect) {
        if let Some((_, current)) = self.floating.iter_mut().find(|(known, _)| *known == window) {
            *current = frame;
        }
    }

    pub fn arrange(&self, screen: Screen, gaps: Gaps) -> Vec<Placement> {
        let focused = self.focused();
        let place = |window: WindowId, frame: Rect, layer: Layer| Placement {
            window,
            frame,
            layer,
            focused: focused == Some(window),
        };
        let tiled = self
            .tiled
            .layout(screen.area)
            .into_iter()
            .map(|(window, tile)| place(window, gaps.apply(tile, screen.area), Layer::Tiled));
        let floating = self.floating.iter().map(|(window, frame)| place(*window, *frame, Layer::Floating));
        let mut placements: Vec<Placement> = tiled.chain(floating).collect();
        if let Some((window, mode)) = self.fullscreen {
            placements.retain(|placement| placement.window != window);
            let frame = match mode {
                Fullscreen::Monitor => screen.full,
                Fullscreen::Maximized => gaps.outer_area(screen.area),
            };
            placements.push(place(window, frame, Layer::Fullscreen(mode)));
        }
        placements
    }
}
