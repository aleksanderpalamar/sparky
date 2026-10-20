use smithay::backend::renderer::element::AsRenderElements;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};
use smithay::desktop::{LayerMap, Window, layer_map_for_output};
use smithay::output::Output;
use smithay::utils::{Logical, Point, Scale};
use smithay::wayland::shell::wlr_layer::Layer as ShellLayer;
use sparky_tiling::{Fullscreen, Layer, Placement, Rect};

use crate::render::border;
use crate::render::elements::SparkyElement;
use crate::state::{BORDER, Sparky};

struct Target {
    origin: Point<i32, Logical>,
    scale: Scale<f64>,
}

pub fn output_elements<R>(state: &mut Sparky, renderer: &mut R, output: &Output) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + 'static,
{
    let Some(geometry) = state.space.output_geometry(output) else {
        return Vec::new();
    };
    let target = Target {
        origin: geometry.loc,
        scale: Scale::from(output.current_scale().fractional_scale()),
    };
    let placements = placements_on(state, &output.name());
    let (above, below): (Vec<&Placement>, Vec<&Placement>) = placements
        .iter()
        .rev()
        .partition(|placement| placement.layer == Layer::Fullscreen(Fullscreen::Monitor));
    let map = layer_map_for_output(output);
    let mut elements = layer_elements(&map, renderer, ShellLayer::Overlay, &target);
    for window in state.unmanaged.iter().rev() {
        elements.extend(surface(&state.space, window, renderer, &target));
    }
    elements.extend(windows(state, renderer, &above, &target));
    elements.extend(layer_elements(&map, renderer, ShellLayer::Top, &target));
    elements.extend(windows(state, renderer, &below, &target));
    elements.extend(layer_elements(&map, renderer, ShellLayer::Bottom, &target));
    elements.extend(layer_elements(&map, renderer, ShellLayer::Background, &target));
    elements
}

fn placements_on(state: &Sparky, output: &str) -> Vec<Placement> {
    state
        .desktop
        .arrange()
        .into_iter()
        .find(|(name, _)| name == output)
        .map(|(_, placements)| placements)
        .unwrap_or_default()
}

fn windows<R>(state: &mut Sparky, renderer: &mut R, placements: &[&Placement], target: &Target) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + 'static,
{
    let mut elements = Vec::new();
    for placement in placements {
        let Some(window) = state.windows.get(placement.window).cloned() else {
            continue;
        };
        elements.extend(surface(&state.space, &window, renderer, target));
        let width = border::border_width(placement, BORDER);
        let frame = relative(placement.frame, target.origin);
        let color = state.colors.for_focus(placement.focused);
        let edges = state.frames.elements(placement.window, frame, width, color, target.scale);
        elements.extend(edges.into_iter().map(SparkyElement::Solid));
    }
    elements
}

fn surface<R>(
    space: &smithay::desktop::Space<Window>,
    window: &Window,
    renderer: &mut R,
    target: &Target,
) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + 'static,
{
    let Some(location) = space.element_location(window) else {
        return Vec::new();
    };
    let origin = location - window.geometry().loc - target.origin;
    window.render_elements(renderer, origin.to_physical_precise_round(target.scale), target.scale, 1.0)
}

fn layer_elements<R>(map: &LayerMap, renderer: &mut R, layer: ShellLayer, target: &Target) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + 'static,
{
    map.layers_on(layer)
        .rev()
        .filter_map(|surface| map.layer_geometry(surface).map(|geometry| (surface, geometry.loc)))
        .flat_map(|(surface, location)| {
            AsRenderElements::<R>::render_elements::<WaylandSurfaceRenderElement<R>>(
                surface,
                renderer,
                location.to_physical_precise_round(target.scale),
                target.scale,
                1.0,
            )
        })
        .map(SparkyElement::Surface)
        .collect()
}

fn relative(frame: Rect, origin: Point<i32, Logical>) -> Rect {
    Rect::new(frame.x - origin.x, frame.y - origin.y, frame.width, frame.height)
}
