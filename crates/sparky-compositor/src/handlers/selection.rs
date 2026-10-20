use std::os::unix::io::OwnedFd;

use smithay::input::Seat;
use smithay::wayland::selection::{SelectionHandler, SelectionSource, SelectionTarget};
use smithay::wayland::selection::data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler};
use smithay::wayland::selection::primary_selection::{PrimarySelectionHandler, PrimarySelectionState};
use smithay::wayland::selection::wlr_data_control::{DataControlHandler, DataControlState};
use smithay::{delegate_data_control, delegate_data_device, delegate_primary_selection};

use crate::state::Sparky;

impl SelectionHandler for Sparky {
    type SelectionUserData = ();

    fn new_selection(&mut self, target: SelectionTarget, source: Option<SelectionSource>, _seat: Seat<Self>) {
        self.wayland_selection_changed(target, source);
    }

    fn send_selection(&mut self, target: SelectionTarget, mime_type: String, fd: OwnedFd, _seat: Seat<Self>, _user_data: &()) {
        self.send_wayland_selection_to_x11(target, mime_type, fd);
    }
}

impl DataDeviceHandler for Sparky {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.protocols.data_device
    }
}

impl ClientDndGrabHandler for Sparky {}

impl ServerDndGrabHandler for Sparky {}

impl PrimarySelectionHandler for Sparky {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.protocols.primary_selection
    }
}

impl DataControlHandler for Sparky {
    fn data_control_state(&self) -> &DataControlState {
        &self.protocols.data_control
    }
}

delegate_data_device!(Sparky);
delegate_primary_selection!(Sparky);
delegate_data_control!(Sparky);
