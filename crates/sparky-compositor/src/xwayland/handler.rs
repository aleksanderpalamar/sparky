use std::os::unix::io::OwnedFd;

use smithay::desktop::Window;
use smithay::wayland::selection::SelectionTarget;
use smithay::utils::{Logical, Rectangle};
use smithay::wayland::xwayland_shell::{XWaylandShellHandler, XWaylandShellState};
use smithay::xwayland::xwm::{Reorder, ResizeEdge, XwmId};
use smithay::xwayland::{X11Surface, X11Wm, XwmHandler};
use sparky_tiling::Fullscreen;
use sparky_tiling::drag_session::Gesture;

use crate::drag::edges::from_x11;
use crate::state::Sparky;
use crate::windows::rules;
use crate::windows::traits::traits_of;

impl XWaylandShellHandler for Sparky {
    fn xwayland_shell_state(&mut self) -> &mut XWaylandShellState {
        &mut self.protocols.xwayland_shell
    }
}

impl XwmHandler for Sparky {
    fn xwm_state(&mut self, _xwm: XwmId) -> &mut X11Wm {
        self.xwm
            .as_mut()
            .expect("o X11Wm só chama o tratador depois de existir")
    }

    fn new_window(&mut self, _xwm: XwmId, _window: X11Surface) {}

    fn new_override_redirect_window(&mut self, _xwm: XwmId, _window: X11Surface) {}

    fn map_window_request(&mut self, _xwm: XwmId, x11: X11Surface) {
        if let Err(error) = x11.set_mapped(true) {
            tracing::warn!("janela X11 não mapeada: {error}");
            return;
        }
        let window = Window::new_x11_window(x11);
        let opening = rules::opening(&traits_of(&window), self.desktop.style(), self.border());
        let id = self.windows.insert(window);
        self.desktop.add_window(id, opening);
        self.relayout();
    }

    fn mapped_override_redirect_window(&mut self, _xwm: XwmId, x11: X11Surface) {
        let location = x11.geometry().loc;
        let window = Window::new_x11_window(x11);
        self.space.map_element(window.clone(), location, false);
        self.unmanaged.push(window);
        self.needs_render = true;
    }

    fn unmapped_window(&mut self, _xwm: XwmId, x11: X11Surface) {
        self.forget_x11(&x11);
        if !x11.is_override_redirect() && x11.set_mapped(false).is_err() {
            tracing::warn!("janela X11 {} não desmapeada", x11.window_id());
        }
    }

    fn destroyed_window(&mut self, _xwm: XwmId, x11: X11Surface) {
        self.forget_x11(&x11);
    }

    fn configure_request(
        &mut self,
        _xwm: XwmId,
        x11: X11Surface,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
        _reorder: Option<Reorder>,
    ) {
        if self.managed_x11(&x11).is_some() {
            self.relayout();
            return;
        }
        let mut geometry = x11.geometry();
        geometry.loc.x = x.unwrap_or(geometry.loc.x);
        geometry.loc.y = y.unwrap_or(geometry.loc.y);
        geometry.size.w = width.map_or(geometry.size.w, |value| value as i32);
        geometry.size.h = height.map_or(geometry.size.h, |value| value as i32);
        if x11.configure(geometry).is_err() {
            tracing::warn!("janela X11 {} recusou o tamanho pedido", x11.window_id());
        }
    }

    fn configure_notify(&mut self, _xwm: XwmId, x11: X11Surface, geometry: Rectangle<i32, Logical>, _above: Option<u32>) {
        let found = self.unmanaged.iter().find(|window| window.x11_surface() == Some(&x11)).cloned();
        if let Some(window) = found {
            self.space.map_element(window, geometry.loc, false);
            self.needs_render = true;
        }
    }

    fn fullscreen_request(&mut self, _xwm: XwmId, x11: X11Surface) {
        self.set_x11_fullscreen(&x11, true);
    }

    fn unfullscreen_request(&mut self, _xwm: XwmId, x11: X11Surface) {
        self.set_x11_fullscreen(&x11, false);
    }

    fn resize_request(&mut self, _xwm: XwmId, x11: X11Surface, _button: u32, edge: ResizeEdge) {
        if let Some(window) = self.managed_x11(&x11) {
            self.start_client_drag(window, Gesture::Resize(from_x11(edge)), None);
        }
    }

    fn move_request(&mut self, _xwm: XwmId, x11: X11Surface, _button: u32) {
        if let Some(window) = self.managed_x11(&x11) {
            self.start_client_drag(window, Gesture::Move, None);
        }
    }

    fn allow_selection_access(&mut self, xwm: XwmId, _selection: SelectionTarget) -> bool {
        self.x11_may_read_selection(xwm)
    }

    fn send_selection(&mut self, _xwm: XwmId, selection: SelectionTarget, mime_type: String, fd: OwnedFd) {
        self.x11_requested_selection(selection, mime_type, fd);
    }

    fn new_selection(&mut self, _xwm: XwmId, selection: SelectionTarget, mime_types: Vec<String>) {
        self.x11_selection_changed(selection, mime_types);
    }

    fn cleared_selection(&mut self, _xwm: XwmId, selection: SelectionTarget) {
        self.x11_selection_cleared(selection);
    }
}

impl Sparky {
    fn managed_x11(&self, x11: &X11Surface) -> Option<sparky_tiling::WindowId> {
        self.windows
            .find(|window| window.x11_surface() == Some(x11))
            .map(|(id, _)| id)
    }

    fn forget_x11(&mut self, x11: &X11Surface) {
        if let Some(id) = self.managed_x11(x11) {
            self.forget_window(id);
            return;
        }
        let found = self.unmanaged.iter().position(|window| window.x11_surface() == Some(x11));
        if let Some(index) = found {
            let window = self.unmanaged.remove(index);
            self.space.unmap_elem(&window);
            self.needs_render = true;
        }
    }

    fn set_x11_fullscreen(&mut self, x11: &X11Surface, wanted: bool) {
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
