use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::idle_inhibit::IdleInhibitHandler;
use smithay::wayland::idle_notify::{IdleNotifierHandler, IdleNotifierState};
use smithay::{delegate_idle_inhibit, delegate_idle_notify};

use crate::state::Sparky;

impl IdleNotifierHandler for Sparky {
    fn idle_notifier_state(&mut self) -> &mut IdleNotifierState<Self> {
        &mut self.idle.notifier
    }
}

impl IdleInhibitHandler for Sparky {
    fn inhibit(&mut self, surface: WlSurface) {
        self.idle.inhibit(surface);
    }

    fn uninhibit(&mut self, surface: WlSurface) {
        self.idle.uninhibit(&surface);
    }
}

delegate_idle_notify!(Sparky);
delegate_idle_inhibit!(Sparky);
