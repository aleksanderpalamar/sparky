use smithay::backend::renderer::utils::with_renderer_surface_state;
use smithay::desktop::{PopupManager, Window, layer_map_for_output};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle};
use smithay::wayland::seat::WaylandFocus;
use sparky_ipc::GlassArea;

use super::area::{radius, rects};
use super::requested::{self, requested};
use crate::state::Sparky;
use crate::windows::traits::traits_of;

const NATIVE: &str = "sparky-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Sparky,
    Other,
}

pub fn owner(name: &str) -> Owner {
    match name.starts_with(NATIVE) {
        true => Owner::Sparky,
        false => Owner::Other,
    }
}

pub struct Pane {
    pub surface: WlSurface,
    pub region: Vec<Rectangle<i32, Logical>>,
    pub radius: i32,
}

pub fn panes(state: &Sparky, output: &Output) -> Vec<Pane> {
    if !state.config.general.frosted_glass {
        return Vec::new();
    }
    let Some(origin) = state
        .space
        .output_geometry(output)
        .map(|geometry| geometry.loc)
    else {
        return Vec::new();
    };
    let mut found = layers(state, output);
    for (_, window) in state.windows.iter() {
        found.extend(window_panes(state, window, output, origin));
    }
    found
}

fn layers(state: &Sparky, output: &Output) -> Vec<Pane> {
    let map = layer_map_for_output(output);
    map.layers()
        .filter_map(|layer| Some((layer, map.layer_geometry(layer)?)))
        .flat_map(|(layer, geometry)| {
            let surface = layer.wl_surface();
            let own = asked(surface, geometry.loc).or_else(|| {
                let area = state.glass.of(layer.namespace())?;
                pane(surface.clone(), &area, geometry)
            });
            let popups = popups(surface, geometry.loc, owner(layer.namespace()));
            own.into_iter().chain(popups)
        })
        .collect()
}

fn window_panes(
    state: &Sparky,
    window: &Window,
    output: &Output,
    origin: Point<i32, Logical>,
) -> Vec<Pane> {
    let Some(surface) = window.wl_surface().map(|surface| surface.into_owned()) else {
        return Vec::new();
    };
    let Some(location) = state.space.element_location(window) else {
        return Vec::new();
    };
    if !state.space.outputs_for_element(window).contains(output) {
        return Vec::new();
    }
    let location = location - origin;
    let by = owner(&traits_of(window).app_id);
    let own = asked(&surface, location - window.geometry().loc).or_else(|| match by {
        Owner::Sparky => pane(
            surface.clone(),
            &GlassArea::Whole,
            Rectangle::new(location, window.geometry().size),
        ),
        Owner::Other => None,
    });
    own.into_iter()
        .chain(popups(&surface, location, by))
        .collect()
}

fn popups(root: &WlSurface, origin: Point<i32, Logical>, by: Owner) -> Vec<Pane> {
    PopupManager::popups_for_surface(root)
        .filter_map(|(popup, offset)| {
            let surface = popup.wl_surface();
            let geometry = popup.geometry();
            asked(surface, origin + offset - geometry.loc).or_else(|| match by {
                Owner::Sparky => pane(
                    surface.clone(),
                    &GlassArea::Whole,
                    Rectangle::new(origin + offset, geometry.size),
                ),
                Owner::Other => None,
            })
        })
        .collect()
}

fn asked(surface: &WlSurface, origin: Point<i32, Logical>) -> Option<Pane> {
    let region = requested(surface)?;
    let size = with_renderer_surface_state(surface, |state| state.surface_size()).flatten()?;
    let region = requested::rects(&region, origin, size);
    (!region.is_empty()).then(|| Pane {
        surface: surface.clone(),
        region,
        radius: 0,
    })
}

fn pane(surface: WlSurface, area: &GlassArea, geometry: Rectangle<i32, Logical>) -> Option<Pane> {
    let region = rects(area, geometry);
    (!region.is_empty()).then_some(Pane {
        surface,
        region,
        radius: radius(area),
    })
}
