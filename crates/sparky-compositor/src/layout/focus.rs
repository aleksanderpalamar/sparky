use smithay::desktop::layer_map_for_output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::shell::wlr_layer::{KeyboardInteractivity, Layer};

use crate::state::Sparky;

impl Sparky {
    pub fn update_keyboard_focus(&mut self) {
        let target = self.exclusive_layer().or_else(|| self.focused_surface());
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }

    fn focused_surface(&self) -> Option<WlSurface> {
        let id = self.desktop.focused_window()?;
        let window = self.windows.get(id)?;
        Some(window.toplevel()?.wl_surface().clone())
    }

    fn exclusive_layer(&self) -> Option<WlSurface> {
        self.space.outputs().find_map(|output| {
            let map = layer_map_for_output(output);
            map.layers()
                .find(|layer| {
                    let state = layer.cached_state();
                    matches!(state.layer, Layer::Top | Layer::Overlay)
                        && state.keyboard_interactivity == KeyboardInteractivity::Exclusive
                })
                .map(|layer| layer.wl_surface().clone())
        })
    }
}
