pub mod rules;

use std::collections::HashMap;

use smithay::reexports::wayland_server::Resource;
use smithay::wayland::session_lock::LockSurface;

use crate::lock::rules::Phase;

#[derive(Default)]
pub struct Lock {
    phase: Phase,
    surfaces: HashMap<String, LockSurface>,
}

impl Lock {
    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_locked(&self) -> bool {
        self.phase == Phase::Locked
    }

    pub fn engage(&mut self) {
        self.phase = Phase::Locked;
        self.surfaces.clear();
    }

    pub fn release(&mut self) {
        self.phase = Phase::Open;
        self.surfaces.clear();
    }

    pub fn attach(&mut self, output: String, surface: LockSurface) {
        self.surfaces.insert(output, surface);
    }

    pub fn surface_on(&self, output: &str) -> Option<&LockSurface> {
        self.surfaces.get(output).filter(|surface| surface.wl_surface().is_alive())
    }

    pub fn live(&self) -> usize {
        self.surfaces.values().filter(|surface| surface.wl_surface().is_alive()).count()
    }
}
