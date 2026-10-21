use smithay::delegate_session_lock;
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::wayland::session_lock::{LockSurface, SessionLockHandler, SessionLockManagerState, SessionLocker};

use crate::lock::rules::accepts;
use crate::state::Sparky;

impl SessionLockHandler for Sparky {
    fn lock_state(&mut self) -> &mut SessionLockManagerState {
        &mut self.protocols.session_lock
    }

    fn lock(&mut self, confirmation: SessionLocker) {
        if !accepts(self.lock.phase(), self.lock.live()) {
            return;
        }
        self.lock.engage();
        confirmation.lock();
        self.lock_changed();
    }

    fn unlock(&mut self) {
        self.lock.release();
        self.lock_changed();
    }

    fn new_surface(&mut self, surface: LockSurface, wl_output: WlOutput) {
        let Some(output) = Output::from_resource(&wl_output) else {
            return;
        };
        let Some(geometry) = self.space.output_geometry(&output) else {
            return;
        };
        surface.with_pending_state(|state| state.size = Some((geometry.size.w as u32, geometry.size.h as u32).into()));
        surface.send_configure();
        self.lock.attach(output.name(), surface);
        self.lock_changed();
    }
}

impl Sparky {
    fn lock_changed(&mut self) {
        self.update_keyboard_focus();
        self.needs_render = true;
    }
}

delegate_session_lock!(Sparky);
