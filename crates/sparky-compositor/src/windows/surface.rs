use smithay::desktop::{Window, WindowSurface};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::get_parent;

use crate::focus::KeyboardFocus;

pub fn focus_target(window: &Window) -> KeyboardFocus {
    match window.underlying_surface() {
        WindowSurface::Wayland(toplevel) => KeyboardFocus::Surface(toplevel.wl_surface().clone()),
        WindowSurface::X11(x11) => KeyboardFocus::X11(x11.clone()),
    }
}

pub fn root_surface(window: &Window) -> Option<WlSurface> {
    match window.underlying_surface() {
        WindowSurface::Wayland(toplevel) => Some(toplevel.wl_surface().clone()),
        WindowSurface::X11(x11) => x11.wl_surface(),
    }
}

pub fn root_of(surface: &WlSurface) -> WlSurface {
    let mut root = surface.clone();
    while let Some(parent) = get_parent(&root) {
        root = parent;
    }
    root
}

pub fn close(window: &Window) {
    match window.underlying_surface() {
        WindowSurface::Wayland(toplevel) => toplevel.send_close(),
        WindowSurface::X11(x11) => {
            if let Err(error) = x11.close() {
                tracing::warn!("janela X11 não fechou: {error}");
            }
        }
    }
}
