use smithay::input::pointer::PointerHandle;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};
use smithay::wayland::pointer_constraints::{PointerConstraintsHandler, with_pointer_constraint};
use smithay::wayland::seat::WaylandFocus;
use smithay::wayland::tablet_manager::TabletSeatHandler;
use smithay::{delegate_cursor_shape, delegate_pointer_constraints, delegate_relative_pointer};

use crate::state::Sparky;

impl TabletSeatHandler for Sparky {}

impl PointerConstraintsHandler for Sparky {
    fn new_constraint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>) {
        if pointer.current_focus().as_ref() != Some(surface) {
            return;
        }
        with_pointer_constraint(surface, pointer, |constraint| {
            if let Some(constraint) = constraint {
                constraint.activate();
            }
        });
    }

    fn cursor_position_hint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>, location: Point<f64, Logical>) {
        let active = with_pointer_constraint(surface, pointer, |constraint| constraint.is_some_and(|constraint| constraint.is_active()));
        if let (true, Some(origin)) = (active, self.surface_origin(surface)) {
            pointer.set_location(origin + location);
        }
    }
}

impl Sparky {
    fn surface_origin(&self, surface: &WlSurface) -> Option<Point<f64, Logical>> {
        let window = self.space.elements().find(|window| window.wl_surface().as_deref() == Some(surface))?;
        let location = self.space.element_location(window)?;
        Some((location - window.geometry().loc).to_f64())
    }
}

delegate_cursor_shape!(Sparky);
delegate_relative_pointer!(Sparky);
delegate_pointer_constraints!(Sparky);
