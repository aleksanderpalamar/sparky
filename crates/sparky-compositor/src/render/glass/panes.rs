use smithay::desktop::{Window, layer_map_for_output};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Rectangle};
use smithay::wayland::seat::WaylandFocus;
use sparky_ipc::GlassArea;

use super::area::rects;
use crate::state::Sparky;
use crate::windows::traits::traits_of;

const NATIVE: &str = "sparky-";

pub struct Pane {
    pub surface: WlSurface,
    pub region: Vec<Rectangle<i32, Logical>>,
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
    found.extend(
        state
            .windows
            .iter()
            .filter_map(|(_, window)| window_pane(state, window, output, origin)),
    );
    found
}

fn layers(state: &Sparky, output: &Output) -> Vec<Pane> {
    let map = layer_map_for_output(output);
    map.layers()
        .filter_map(|layer| {
            let area = state.glass.of(layer.namespace())?;
            let geometry = map.layer_geometry(layer)?;
            pane(layer.wl_surface().clone(), &area, geometry)
        })
        .collect()
}

fn window_pane(
    state: &Sparky,
    window: &Window,
    output: &Output,
    origin: smithay::utils::Point<i32, Logical>,
) -> Option<Pane> {
    let surface = window.wl_surface()?.into_owned();
    if !traits_of(window).app_id.starts_with(NATIVE)
        || !state.space.outputs_for_element(window).contains(output)
    {
        return None;
    }
    let location = state.space.element_location(window)? - origin;
    pane(
        surface,
        &GlassArea::Whole,
        Rectangle::new(location, window.geometry().size),
    )
}

fn pane(surface: WlSurface, area: &GlassArea, geometry: Rectangle<i32, Logical>) -> Option<Pane> {
    let region = rects(area, geometry);
    (!region.is_empty()).then_some(Pane { surface, region })
}
