use super::formats::Formats;
use super::queue::{Job, Notify};
use super::types::Session;
use smithay::reexports::calloop::RegistrationToken;
use smithay::reexports::wayland_server::Weak;
use smithay::wayland::foreign_toplevel_list::ForeignToplevelHandle;
use sparky_tiling::WindowId;
use std::collections::HashMap;
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
pub struct ImageCaptureState {
    pub formats: Formats,
    pub toplevels: HashMap<WindowId, ForeignToplevelHandle>,
    pub sessions: Vec<Weak<Session>>,
    pub pending: Vec<Job>,
    pub flights: Vec<Flight>,
    pub generation: u64,
    pub wake: Option<Wake>,
    pub source_timer: Option<RegistrationToken>,
    pub cursors: Option<crate::backend::drm::cursor::Cursors>,
}

pub struct Flight {
    pub token: RegistrationToken,
    pub notify: Notify,
    pub done: Rc<Cell<bool>>,
}

pub struct Wake {
    pub token: RegistrationToken,
    pub deadline: std::time::Duration,
}

impl ImageCaptureState {
    pub fn forget(&mut self, id: WindowId) {
        if let Some(handle) = self.toplevels.remove(&id) {
            handle.send_closed();
        }
    }
}
