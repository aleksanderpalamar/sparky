use smithay::desktop::{WindowSurfaceType, layer_map_for_output};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};
use smithay::wayland::shell::wlr_layer::Layer;
use sparky_tiling::WindowId;

use crate::layout::stacking::{Cover, Stratum, strata};
use crate::state::Sparky;
use crate::windows::surface::{root_of, root_surface};

pub type Under = (WlSurface, Point<f64, Logical>);

impl Sparky {
    pub fn surface_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        if self.lock.is_locked() {
            return self.lock_under(position);
        }
        let cover = self
            .space
            .output_under(position)
            .next()
            .map_or(Cover::Open, |output| self.stacking.cover(&output.name()));
        strata(cover)
            .iter()
            .find_map(|stratum| self.stratum_under(*stratum, position))
    }

    pub fn window_under(&self, position: Point<f64, Logical>) -> Option<WindowId> {
        if self.lock.is_locked() {
            return None;
        }
        let (window, _) = self.space.element_under(position)?;
        let (id, _) = self.windows.find(|known| known == window)?;
        (!self.desktop.is_covered(id)).then_some(id)
    }

    pub fn window_owning(&self, surface: &WlSurface) -> Option<WindowId> {
        let root = root_of(surface);
        let (id, _) = self
            .windows
            .find(|window| root_surface(window).as_ref() == Some(&root))?;
        (!self.desktop.is_covered(id)).then_some(id)
    }

    fn stratum_under(&self, stratum: Stratum, position: Point<f64, Logical>) -> Option<Under> {
        match stratum {
            Stratum::Overlay => self.layer_under(position, Layer::Overlay),
            Stratum::Unmanaged => self.unmanaged_under(position),
            Stratum::Top => self.layer_under(position, Layer::Top),
            Stratum::Windows => self.window_surface_under(position),
            Stratum::Bottom => self.layer_under(position, Layer::Bottom),
            Stratum::Background => self.layer_under(position, Layer::Background),
        }
    }

    fn lock_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        let output = self.space.output_under(position).next()?;
        let origin = self.space.output_geometry(output)?.loc;
        let surface = self.lock.surface_on(&output.name())?;
        Some((surface.wl_surface().clone(), origin.to_f64()))
    }

    fn unmanaged_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        let (window, _) = self.space.element_under(position)?;
        if !self.unmanaged.contains(window) {
            return None;
        }
        self.window_surface_under(position)
    }

    fn window_surface_under(&self, position: Point<f64, Logical>) -> Option<Under> {
        let (window, location) = self.space.element_under(position)?;
        let (surface, offset) =
            window.surface_under(position - location.to_f64(), WindowSurfaceType::ALL)?;
        Some((surface, (offset + location).to_f64()))
    }

    fn layer_under(&self, position: Point<f64, Logical>, layer: Layer) -> Option<Under> {
        let output = self.space.output_under(position).next()?;
        let origin = self.space.output_geometry(output)?.loc;
        let map = layer_map_for_output(output);
        let local = position - origin.to_f64();
        let surface = map.layer_under(layer, local)?;
        let geometry = map.layer_geometry(surface)?;
        let (found, offset) =
            surface.surface_under(local - geometry.loc.to_f64(), WindowSurfaceType::ALL)?;
        Some((found, (offset + geometry.loc + origin).to_f64()))
    }
}
