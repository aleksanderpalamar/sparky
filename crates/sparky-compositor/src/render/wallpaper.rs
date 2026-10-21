use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};
use smithay::output::Output;
use smithay::utils::{Logical, Point, Size};

use crate::render::elements::SparkyElement;
use crate::state::Sparky;

pub fn wallpaper_element<R>(state: &Sparky, renderer: &mut R, output: &Output, size: Size<i32, Logical>) -> Option<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + Send + 'static,
{
    let buffer = state.wallpaper.buffer(&output.name())?;
    let placed = MemoryRenderBufferRenderElement::from_buffer(renderer, Point::from((0.0, 0.0)), buffer, None, None, Some(size), Kind::Unspecified);
    placed.ok().map(SparkyElement::Memory)
}
