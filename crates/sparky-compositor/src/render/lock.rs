use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::surface::render_elements_from_surface_tree;
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};
use smithay::output::Output;
use smithay::utils::{Physical, Point, Scale};

use crate::render::elements::SparkyElement;
use crate::state::Sparky;

pub fn lock_elements<R>(state: &Sparky, renderer: &mut R, output: &Output, scale: Scale<f64>) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + Send + 'static,
{
    let Some(surface) = state.lock.surface_on(&output.name()) else {
        return Vec::new();
    };
    let origin: Point<i32, Physical> = (0, 0).into();
    render_elements_from_surface_tree(renderer, surface.wl_surface(), origin, scale, 1.0, Kind::Unspecified)
}
