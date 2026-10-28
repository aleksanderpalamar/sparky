use std::collections::{HashMap, HashSet};
use std::time::Instant;

use iced::window;

use crate::domain::reveal::{Expansion, Reveal};

#[derive(Debug, Default)]
pub struct Drawers {
    windows: HashMap<window::Id, Reveal>,
    hovered: HashSet<window::Id>,
}

impl Drawers {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn progress(&self, window: window::Id) -> f32 {
        self.windows.get(&window).map_or(0.0, Reveal::progress)
    }

    pub fn animating(&self) -> bool {
        self.windows.values().any(Reveal::animating)
    }

    pub fn hover(
        &mut self,
        window: window::Id,
        expansion: Expansion,
        menu_open: bool,
        now: Instant,
    ) {
        match expansion {
            Expansion::Open => {
                self.hovered.insert(window);
            }
            Expansion::Closed => {
                self.hovered.remove(&window);
            }
        }
        if expansion == Expansion::Closed && menu_open {
            return;
        }
        self.windows.entry(window).or_default().set(expansion, now);
    }

    pub fn release(&mut self, now: Instant) {
        for (window, reveal) in &mut self.windows {
            let expansion = match self.hovered.contains(window) {
                true => Expansion::Open,
                false => Expansion::Closed,
            };
            reveal.set(expansion, now);
        }
    }

    pub fn advance(&mut self, now: Instant) {
        for reveal in self.windows.values_mut() {
            reveal.advance(now);
        }
        self.windows
            .retain(|_, reveal| reveal.animating() || reveal.target() == Expansion::Open);
    }
}
