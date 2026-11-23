use std::os::unix::io::OwnedFd;

use smithay::input::Seat;
use smithay::input::dnd::{DnDGrab, DndGrabHandler, GrabType, Source};
use smithay::input::pointer::Focus;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;
use smithay::wayland::selection::data_device::{
    DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler,
};
use smithay::wayland::selection::primary_selection::{
    PrimarySelectionHandler, PrimarySelectionState,
};
use smithay::wayland::selection::wlr_data_control::{DataControlHandler, DataControlState};
use smithay::wayland::selection::{SelectionHandler, SelectionSource, SelectionTarget};

use crate::state::Sparky;

impl SelectionHandler for Sparky {
    type SelectionUserData = ();

    fn new_selection(
        &mut self,
        target: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        self.wayland_selection_changed(target, source);
    }

    fn send_selection(
        &mut self,
        target: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        _user_data: &(),
    ) {
        self.send_wayland_selection_to_x11(target, mime_type, fd);
    }
}

impl DataDeviceHandler for Sparky {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.protocols.data_device
    }
}

impl WaylandDndGrabHandler for Sparky {
    fn dnd_requested<S: Source>(
        &mut self,
        source: S,
        _icon: Option<WlSurface>,
        seat: Seat<Self>,
        serial: Serial,
        type_: GrabType,
    ) {
        match type_ {
            GrabType::Pointer => {
                let Some(pointer) = seat.get_pointer() else {
                    return source.cancel();
                };
                let Some(start) = pointer.grab_start_data() else {
                    return source.cancel();
                };
                let grab = DnDGrab::new_pointer(&self.display_handle, start, source, seat);
                pointer.set_grab(self, grab, serial, Focus::Keep);
            }
            GrabType::Touch => {
                let Some(touch) = seat.get_touch() else {
                    return source.cancel();
                };
                let Some(start) = touch.grab_start_data() else {
                    return source.cancel();
                };
                let grab = DnDGrab::new_touch(&self.display_handle, start, source, seat);
                touch.set_grab(self, grab, serial);
            }
        }
    }
}

impl DndGrabHandler for Sparky {}

impl PrimarySelectionHandler for Sparky {
    fn primary_selection_state(&mut self) -> &mut PrimarySelectionState {
        &mut self.protocols.primary_selection
    }
}

impl DataControlHandler for Sparky {
    fn data_control_state(&mut self) -> &mut DataControlState {
        &mut self.protocols.data_control
    }
}
