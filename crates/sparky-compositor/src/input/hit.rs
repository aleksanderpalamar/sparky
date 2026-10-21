use smithay::desktop::{WindowSurfaceType, layer_map_for_output};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};
use smithay::wayland::shell::wlr_layer::Layer;
use sparky_tiling::WindowId;

use crate::state::Sparky;

pub type Under = (WlSurface, Point<f64, Logical>);

impl Sparky {
    pub fn surface_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        if self.lock.is_locked() {
            return self.lock_under(position);
        }
        self.layer_under(position, &[Layer::Overlay, Layer::Top])
            .or_else(|| self.window_surface_under(position))
            .or_else(|| self.layer_under(position, &[Layer::Bottom, Layer::Background]))
    }

    pub fn window_under(&self, position: Point<f64, Logical>) -> Option<WindowId> {
        if self.lock.is_locked() {
            return None;
        }
        let (window, _) = self.space.element_under(position)?;
        let (id, _) = self.windows.find(|known| known == window)?;
        (!self.desktop.is_covered(id)).then_some(id)
    }

    fn lock_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        let output = self.space.output_under(position).next()?;
        let origin = self.space.output_geometry(output)?.loc;
        let surface = self.lock.surface_on(&output.name())?;
        Some((surface.wl_surface().clone(), origin.to_f64()))
    }

    fn window_surface_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        let (window, location) = self.space.element_under(position)?;
        let (surface, offset) = window.surface_under(position - location.to_f64(), WindowSurfaceType::ALL)?;
        Some((surface, (offset + location).to_f64()))
    }

    fn layer_under(&self, position: Point<f64, Logical>, layers: &[Layer]) -> Option<Under> {
        let output = self.space.output_under(position).next()?;
        let origin = self.space.output_geometry(output)?.loc;
        let map = layer_map_for_output(output);
        let local = position - origin.to_f64();
        layers.iter().find_map(|layer| {
            let surface = map.layer_under(*layer, local)?;
            let geometry = map.layer_geometry(surface)?;
            let (found, offset) = surface.surface_under(local - geometry.loc.to_f64(), WindowSurfaceType::ALL)?;
            Some((found, (offset + geometry.loc + origin).to_f64()))
        })
    }
}
