use smithay::desktop::Window;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;
use smithay::wayland::shell::xdg::ToplevelState;
use sparky_tiling::{Fullscreen, Layer, Placement, WindowId};

use crate::render::border;
use crate::state::{BORDER, Sparky};

const TILED: [State; 4] = [State::TiledLeft, State::TiledRight, State::TiledTop, State::TiledBottom];

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
        self.update_keyboard_focus();
    }

    fn show(&mut self, window: Window, placement: &Placement) {
        let content = border::content(placement.frame, border::border_width(placement, BORDER));
        configure(&window, placement, (content.width, content.height));
        self.space.map_element(window, (content.x, content.y), false);
    }
}

fn configure(window: &Window, placement: &Placement, size: (i32, i32)) {
    let Some(toplevel) = window.toplevel() else {
        return;
    };
    window.set_activated(placement.focused);
    toplevel.with_pending_state(|state| {
        state.size = Some(size.into());
        mark(state, State::Fullscreen, placement.layer == Layer::Fullscreen(Fullscreen::Monitor));
        mark(state, State::Maximized, placement.layer == Layer::Fullscreen(Fullscreen::Maximized));
        for tiled in TILED {
            mark(state, tiled, placement.layer == Layer::Tiled);
        }
    });
    match toplevel.is_initial_configure_sent() {
        true => {
            toplevel.send_pending_configure();
        }
        false => {
            toplevel.send_configure();
        }
    }
}

fn mark(state: &mut ToplevelState, flag: State, on: bool) {
    match on {
        true => state.states.set(flag),
        false => state.states.unset(flag),
    };
}
