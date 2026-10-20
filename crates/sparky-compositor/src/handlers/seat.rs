use smithay::delegate_seat;
use smithay::input::pointer::CursorImageStatus;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::reexports::wayland_server::Resource;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::selection::data_device::set_data_device_focus;
use smithay::wayland::selection::primary_selection::set_primary_focus;

use crate::state::Sparky;

impl SeatHandler for Sparky {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Sparky> {
        &mut self.protocols.seat
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image;
        self.needs_render = true;
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let display = &self.display_handle;
        let client = focused.and_then(|surface| display.get_client(surface.id()).ok());
        set_data_device_focus(display, seat, client.clone());
        set_primary_focus(display, seat, client);
    }
}

delegate_seat!(Sparky);
