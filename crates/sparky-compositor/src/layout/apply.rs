use smithay::desktop::Window;
use sparky_tiling::{Placement, WindowId};

use crate::render::border;
use crate::state::{BORDER, Sparky};
use crate::windows::configure::configure;

impl Sparky {
    pub fn relayout(&mut self) {
        let mut visible: Vec<WindowId> = Vec::new();
        for (_, placements) in self.desktop.arrange() {
            for placement in placements {
                if let Some(window) = self.windows.get(placement.window).cloned() {
                    self.show(window, &placement);
                    visible.push(placement.window);
                }
            }
        }
        let hidden: Vec<Window> = self
            .windows
            .iter()
            .filter(|(id, _)| !visible.contains(id))
            .map(|(_, window)| window.clone())
            .collect();
        for window in hidden {
            self.space.unmap_elem(&window);
        }
        for window in &self.unmanaged {
            self.space.raise_element(window, false);
        }
        self.update_keyboard_focus();
        self.needs_render = true;
    }

    fn show(&mut self, window: Window, placement: &Placement) {
        let content = border::content(placement.frame, border::border_width(placement, BORDER));
        configure(&window, placement, content);
        self.space.map_element(window, (content.x, content.y), false);
    }
}
