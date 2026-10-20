use std::os::unix::io::OwnedFd;

use smithay::input::Seat;
use smithay::wayland::selection::data_device::{
    clear_data_device_selection, current_data_device_selection_userdata, request_data_device_client_selection,
    set_data_device_selection,
};
use smithay::wayland::selection::primary_selection::{
    clear_primary_selection, current_primary_selection_userdata, request_primary_client_selection,
    set_primary_selection,
};
use smithay::wayland::selection::{SelectionSource, SelectionTarget};
use smithay::xwayland::xwm::XwmId;

use crate::focus::KeyboardFocus;
use crate::state::Sparky;

impl Sparky {
    pub fn wayland_selection_changed(&mut self, target: SelectionTarget, source: Option<SelectionSource>) {
        let Some(xwm) = self.xwm.as_mut() else {
            return;
        };
        if let Err(error) = xwm.new_selection(target, source.map(|source| source.mime_types())) {
            tracing::warn!("área de transferência não chegou ao X11: {error}");
        }
    }

    pub fn send_wayland_selection_to_x11(&mut self, target: SelectionTarget, mime_type: String, fd: OwnedFd) {
        let handle = self.loop_handle.clone();
        let Some(xwm) = self.xwm.as_mut() else {
            return;
        };
        if let Err(error) = xwm.send_selection(target, mime_type, fd, handle) {
            tracing::warn!("área de transferência do X11 não enviada: {error}");
        }
    }

    pub fn x11_may_read_selection(&self, xwm: XwmId) -> bool {
        let focus = self.seat.get_keyboard().and_then(|keyboard| keyboard.current_focus());
        matches!(focus, Some(KeyboardFocus::X11(window)) if window.xwm_id() == Some(xwm))
    }

    pub fn x11_requested_selection(&mut self, target: SelectionTarget, mime_type: String, fd: OwnedFd) {
        let result = match target {
            SelectionTarget::Clipboard => request_data_device_client_selection(&self.seat, mime_type, fd).is_ok(),
            SelectionTarget::Primary => request_primary_client_selection(&self.seat, mime_type, fd).is_ok(),
        };
        if !result {
            tracing::warn!("o X11 pediu uma área de transferência que não existe");
        }
    }

    pub fn x11_selection_changed(&mut self, target: SelectionTarget, mime_types: Vec<String>) {
        match target {
            SelectionTarget::Clipboard => set_data_device_selection(&self.display_handle, &self.seat, mime_types, ()),
            SelectionTarget::Primary => set_primary_selection(&self.display_handle, &self.seat, mime_types, ()),
        }
    }

    pub fn x11_selection_cleared(&mut self, target: SelectionTarget) {
        let seat: &Seat<Sparky> = &self.seat;
        match target {
            SelectionTarget::Clipboard if current_data_device_selection_userdata(seat).is_some() => {
                clear_data_device_selection(&self.display_handle, seat)
            }
            SelectionTarget::Primary if current_primary_selection_userdata(seat).is_some() => {
                clear_primary_selection(&self.display_handle, seat)
            }
            _ => {}
        }
    }
}
