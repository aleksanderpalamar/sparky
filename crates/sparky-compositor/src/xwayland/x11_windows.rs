use smithay::xwayland::X11Surface;
use sparky_tiling::Fullscreen;

use super::active::{Minimize, minimize_answer};
use crate::state::Sparky;

impl Sparky {
    pub(super) fn managed_x11(&self, x11: &X11Surface) -> Option<sparky_tiling::WindowId> {
        self.windows
            .find(|window| window.x11_surface() == Some(x11))
            .map(|(id, _)| id)
    }

    pub(super) fn forget_x11(&mut self, x11: &X11Surface) {
        if let Some(id) = self.managed_x11(x11) {
            self.forget_window(id);
            return;
        }
        let found = self
            .unmanaged
            .iter()
            .position(|window| window.x11_surface() == Some(x11));
        if let Some(index) = found {
            let window = self.unmanaged.remove(index);
            self.space.unmap_elem(&window);
            self.needs_render = true;
        }
    }

    pub(super) fn answer_minimize(&mut self, x11: &X11Surface) {
        let Some(id) = self.managed_x11(x11) else {
            return;
        };
        if minimize_answer(id, self.desktop.focused_window()) == Minimize::Refuse {
            self.restore_x11(x11);
        }
    }

    pub(super) fn restore_x11(&mut self, x11: &X11Surface) {
        if x11.set_mapped(true).is_err() {
            tracing::warn!("janela X11 {} não voltou ao estado normal", x11.window_id());
        }
        self.relayout();
    }

    pub(super) fn set_x11_fullscreen(&mut self, x11: &X11Surface, wanted: bool) {
        let Some(id) = self.managed_x11(x11) else {
            return;
        };
        self.desktop.focus_window(id);
        if x11.is_fullscreen() != wanted {
            self.desktop.toggle_fullscreen(Fullscreen::Monitor);
        }
        self.relayout();
    }
}
