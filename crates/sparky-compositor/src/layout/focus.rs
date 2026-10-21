use smithay::desktop::layer_map_for_output;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::shell::wlr_layer::{KeyboardInteractivity, Layer};

use crate::focus::KeyboardFocus;
use crate::state::Sparky;
use crate::windows::surface::focus_target;

impl Sparky {
    pub fn update_keyboard_focus(&mut self) {
        let target = match self.lock.is_locked() {
            true => self.lock_surface(),
            false => self.exclusive_layer().or_else(|| self.focused_surface()),
        };
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }

    fn lock_surface(&self) -> Option<KeyboardFocus> {
        let focused = self.desktop.focused_monitor().map(|monitor| monitor.name.clone());
        let output = focused.into_iter().chain(self.space.outputs().map(|output| output.name()));
        output
            .filter_map(|name| self.lock.surface_on(&name))
            .map(|surface| KeyboardFocus::Surface(surface.wl_surface().clone()))
            .next()
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
