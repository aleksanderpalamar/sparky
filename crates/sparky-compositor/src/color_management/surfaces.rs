use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Resource, Weak};
use smithay::wayland::compositor::with_states;

use super::conversion::{Conversion, conversion};
use super::surface_color::SurfaceColor;

#[derive(Default)]
pub struct Surfaces {
    live: Vec<Weak<WlSurface>>,
}

impl Surfaces {
    pub fn add(&mut self, surface: &WlSurface) {
        self.live.retain(|known| known.id() != surface.id());
        self.live.push(surface.downgrade());
    }

    pub fn prune(&mut self) {
        self.live.retain(|known| known.upgrade().is_ok());
    }

    pub fn conversions(&self) -> Vec<(WlSurface, Conversion)> {
        self.live
            .iter()
            .filter_map(|known| known.upgrade().ok())
            .filter_map(|surface| converted(&surface).map(|found| (surface, found)))
            .collect()
    }
}

fn converted(surface: &WlSurface) -> Option<Conversion> {
    let record = with_states(surface, |states| {
        states
            .cached_state
            .get::<SurfaceColor>()
            .current()
            .description
            .clone()
    })?;
    conversion(&record.description)
}
