use sparky_tiling::WindowId;

use crate::state::Sparky;

impl Sparky {
    pub fn forget_window(&mut self, id: WindowId) {
        if let Some(window) = self.windows.remove(id) {
            self.space.unmap_elem(&window);
        }
        self.frames.forget(id);
        self.desktop.remove_window(id);
        self.relayout();
    }
}
