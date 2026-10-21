use smithay::delegate_fractional_scale;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::fractional_scale::FractionalScaleHandler;

use crate::state::Sparky;

impl FractionalScaleHandler for Sparky {
    fn new_fractional_scale(&mut self, _surface: WlSurface) {
        self.publish_scales();
    }
}

delegate_fractional_scale!(Sparky);
