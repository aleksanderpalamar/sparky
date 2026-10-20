use smithay::desktop::layer_map_for_output;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::shell::wlr_layer::{KeyboardInteractivity, Layer};

use crate::focus::KeyboardFocus;
use crate::state::Sparky;
use crate::windows::surface::focus_target;

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

    fn focused_surface(&self) -> Option<KeyboardFocus> {
        let id = self.desktop.focused_window()?;
        self.windows.get(id).map(focus_target)
    }

    fn exclusive_layer(&self) -> Option<KeyboardFocus> {
        self.space.outputs().find_map(|output| {
            let map = layer_map_for_output(output);
            map.layers()
                .find(|layer| {
                    let state = layer.cached_state();
                    matches!(state.layer, Layer::Top | Layer::Overlay)
                        && state.keyboard_interactivity == KeyboardInteractivity::Exclusive
                })
                .map(|layer| KeyboardFocus::Surface(layer.wl_surface().clone()))
        })
    }
}
