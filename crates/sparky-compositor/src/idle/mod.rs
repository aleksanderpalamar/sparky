pub mod rules;
mod watch;

use std::time::{Duration, Instant};

use smithay::input::Seat;
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::Resource;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::idle_notify::IdleNotifierState;

use crate::idle::rules::Inhibition;
use crate::state::Sparky;

pub struct Idle {
    pub notifier: IdleNotifierState<Sparky>,
    inhibitors: Vec<WlSurface>,
    last_activity: Instant,
}

impl Idle {
    pub fn new(display: &DisplayHandle, loop_handle: LoopHandle<'static, Sparky>) -> Self {
        Self {
            notifier: IdleNotifierState::new(display, loop_handle),
            inhibitors: Vec::new(),
            last_activity: Instant::now(),
        }
    }

    pub fn touch(&mut self, seat: &Seat<Sparky>) {
        self.notifier.notify_activity(seat);
        self.restart();
    }

    pub fn restart(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn idle_for(&self) -> Duration {
        self.last_activity.elapsed()
    }

    pub fn inhibit(&mut self, surface: WlSurface) {
        if !self.inhibitors.contains(&surface) {
            self.inhibitors.push(surface);
        }
        self.sync();
    }

    pub fn uninhibit(&mut self, surface: &WlSurface) {
        self.inhibitors.retain(|inhibitor| inhibitor != surface);
        self.sync();
    }

    pub fn inhibition(&mut self) -> Inhibition {
        self.inhibitors.retain(Resource::is_alive);
        self.sync();
        match self.inhibitors.is_empty() {
            true => Inhibition::Free,
            false => Inhibition::Inhibited,
        }
    }

    fn sync(&mut self) {
        self.notifier.set_is_inhibited(!self.inhibitors.is_empty());
    }
}
