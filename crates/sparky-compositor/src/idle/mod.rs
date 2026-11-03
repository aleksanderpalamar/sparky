pub mod rules;
mod watch;

use std::time::{Duration, Instant};

use smithay::input::Seat;
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::Resource;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::idle_notify::IdleNotifierState;
use sparky_ipc::IdleHold;

use crate::idle::rules::Inhibition;
use crate::state::Sparky;

pub struct Idle {
    pub notifier: IdleNotifierState<Sparky>,
    inhibitors: Vec<WlSurface>,
    holders: usize,
    last_activity: Instant,
}

impl Idle {
    pub fn new(display: &DisplayHandle, loop_handle: LoopHandle<'static, Sparky>) -> Self {
        Self {
            notifier: IdleNotifierState::new(display, loop_handle),
            inhibitors: Vec::new(),
            holders: 0,
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
        self.current()
    }

    pub fn hold(&mut self, from: IdleHold, to: IdleHold) {
        self.holders = match (from, to) {
            (IdleHold::Free, IdleHold::Held) => self.holders.saturating_add(1),
            (IdleHold::Held, IdleHold::Free) => self.holders.saturating_sub(1),
            (IdleHold::Free, IdleHold::Free) | (IdleHold::Held, IdleHold::Held) => self.holders,
        };
        self.sync();
    }

    fn current(&self) -> Inhibition {
        match (self.holders, self.inhibitors.is_empty()) {
            (0, true) => Inhibition::Free,
            _ => Inhibition::Inhibited,
        }
    }

    fn sync(&mut self) {
        let inhibited = self.current() == Inhibition::Inhibited;
        self.notifier.set_is_inhibited(inhibited);
    }
}
