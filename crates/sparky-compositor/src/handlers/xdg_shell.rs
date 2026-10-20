use smithay::delegate_xdg_shell;
use smithay::desktop::{PopupKind, Window, find_popup_root_surface, get_popup_toplevel_coords};
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::utils::Serial;
use smithay::wayland::shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState};
use sparky_tiling::{Fullscreen, WindowId};

use crate::state::Sparky;

impl XdgShellHandler for Sparky {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.protocols.xdg_shell
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        self.windows.insert(Window::new_wayland_window(surface));
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(id) = self.window_of(&surface) else {
            return;
        };
        if let Some(window) = self.windows.remove(id) {
            self.space.unmap_elem(&window);
        }
        self.frames.forget(id);
        self.desktop.remove_window(id);
        self.relayout();
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        if let Err(error) = self.popups.track_popup(PopupKind::Xdg(surface)) {
            tracing::warn!("menu não acompanhado: {error}");
        }
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {}

    fn fullscreen_request(&mut self, surface: ToplevelSurface, _output: Option<WlOutput>) {
        self.set_fullscreen(&surface, true);
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        self.set_fullscreen(&surface, false);
    }
}

impl Sparky {
    pub fn window_of(&self, surface: &ToplevelSurface) -> Option<WindowId> {
        self.windows
            .find(|window| window.toplevel() == Some(surface))
            .map(|(id, _)| id)
    }

    fn set_fullscreen(&mut self, surface: &ToplevelSurface, wanted: bool) {
        let Some(id) = self.window_of(surface) else {
            return;
        };
        let current = surface.with_pending_state(|state| {
            state
                .states
                .contains(smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Fullscreen)
        });
        self.desktop.focus_window(id);
        if current != wanted {
            self.desktop.toggle_fullscreen(Fullscreen::Monitor);
        }
        self.relayout();
    }

    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let Ok(root) = find_popup_root_surface(&PopupKind::Xdg(popup.clone())) else {
            return;
        };
        let Some((_, window)) = self
            .windows
            .find(|window| window.toplevel().is_some_and(|toplevel| toplevel.wl_surface() == &root))
        else {
            return;
        };
        let (Some(window_geometry), Some(output)) = (self.space.element_geometry(window), self.space.outputs().next()) else {
            return;
        };
        let Some(mut target) = self.space.output_geometry(output) else {
            return;
        };
        target.loc -= get_popup_toplevel_coords(&PopupKind::Xdg(popup.clone()));
        target.loc -= window_geometry.loc;
        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}

delegate_xdg_shell!(Sparky);
