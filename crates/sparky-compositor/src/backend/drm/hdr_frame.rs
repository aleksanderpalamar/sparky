use smithay::backend::drm::compositor::FrameFlags;
use smithay::backend::renderer::Color32F;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexProgram};

use crate::backend::drm::gpu::Surface;
use crate::render::elements::SparkyElement;
use crate::render::hdr::composite::Composite;
use crate::render::hdr::pq;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub fn program(renderer: &mut GlesRenderer) -> Option<GlesTexProgram> {
    match renderer.compile_custom_texture_shader(pq::shader(), &[]) {
        Ok(program) => Some(program),
        Err(error) => {
            tracing::warn!("conversão para HDR indisponível: {error}");
            None
        }
    }
}

pub fn render(
    renderer: &mut GlesRenderer,
    surface: &mut Surface,
    elements: &[SparkyElement<GlesRenderer>],
    clear: Color32F,
    program: &GlesTexProgram,
) -> Result<bool, String> {
    if surface
        .composite
        .as_ref()
        .is_none_or(|composite| !composite.fits(&surface.output))
    {
        surface.composite = Some(Composite::new(renderer, &surface.output)?);
    }
    let Some(composite) = surface.composite.as_mut() else {
        return Err("composição HDR indisponível".to_owned());
    };
    composite.draw(renderer, elements, clear)?;
    let element = composite.element(program);
    surface
        .drm_output
        .render_frame(renderer, &[element], BLACK, FrameFlags::empty())
        .map(|result| result.is_empty)
        .map_err(|error| error.to_string())
}
