use std::sync::Mutex;

use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement};
use smithay::backend::renderer::element::surface::render_elements_from_surface_tree;
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};
use smithay::input::pointer::{CursorImageAttributes, CursorImageStatus};
use smithay::utils::{Logical, Point, Scale};
use smithay::wayland::compositor::with_states;

use crate::render::elements::SparkyElement;

pub struct Pointer<'a> {
    pub status: &'a CursorImageStatus,
    pub image: &'a MemoryRenderBuffer,
    pub hotspot: Point<i32, Logical>,
    pub location: Point<f64, Logical>,
}

pub fn cursor_elements<R>(renderer: &mut R, pointer: &Pointer<'_>, scale: Scale<f64>) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + Send + 'static,
{
    match pointer.status {
        CursorImageStatus::Hidden => Vec::new(),
        CursorImageStatus::Surface(surface) => {
            let hotspot = with_states(surface, |states| {
                states
                    .data_map
                    .get::<Mutex<CursorImageAttributes>>()
                    .and_then(|attributes| attributes.lock().ok().map(|attributes| attributes.hotspot))
                    .unwrap_or_default()
            });
            let origin = (pointer.location - hotspot.to_f64()).to_physical_precise_round(scale);
            render_elements_from_surface_tree(renderer, surface, origin, scale, 1.0, Kind::Cursor)
        }
        CursorImageStatus::Named(_) => themed(renderer, pointer, scale),
    }
}

fn themed<R>(renderer: &mut R, pointer: &Pointer<'_>, scale: Scale<f64>) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + Send + 'static,
{
    let origin = (pointer.location - pointer.hotspot.to_f64()).to_physical(scale);
    match MemoryRenderBufferRenderElement::from_buffer(renderer, origin, pointer.image, None, None, None, Kind::Cursor) {
        Ok(element) => vec![SparkyElement::Memory(element)],
        Err(_) => Vec::new(),
    }
}
