use super::queue::Job;
use super::types::Target;
use crate::backend::drm::cursor::Cursors;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;
use smithay::backend::allocator::Buffer;
use smithay::backend::renderer::Bind;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::{
    AsRenderElements,
    utils::{Relocate, RelocateRenderElement},
};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::sync::SyncPoint;
use smithay::utils::{Point, Scale, Transform};

pub fn render(
    state: &mut Sparky,
    renderer: &mut GlesRenderer,
    cursors: &mut Cursors,
    job: &mut Job,
) -> Result<SyncPoint, String> {
    let locked = state.lock.is_locked();
    job.locked = locked;
    let (mut elements, scale, origin) = match &job.target {
        Target::Output(weak) => {
            let output = weak.upgrade().ok_or("monitor removido")?;
            let geometry = state
                .space
                .output_geometry(&output)
                .ok_or("monitor sem geometria")?;
            let scale = Scale::from(output.current_scale().fractional_scale());
            let elements = match locked {
                true => Vec::new(),
                false => crate::render::output::output_elements(state, renderer, &output),
            };
            (elements, scale, geometry.loc)
        }
        Target::Window(id) => {
            let window = state.windows.get(*id).ok_or("janela fechada")?;
            let scale = Scale::from(
                state
                    .space
                    .outputs_for_element(window)
                    .first()
                    .map_or(1.0, |out| out.current_scale().fractional_scale()),
            );
            let geometry = window.geometry();
            let location: Point<i32, _> = geometry.loc.to_physical_precise_round(scale);
            let elements: Vec<SparkyElement<_>> = match locked {
                true => Vec::new(),
                false => {
                    window.render_elements(renderer, (-location.x, -location.y).into(), scale, 1.0)
                }
            };
            (
                elements,
                scale,
                state.space.element_location(window).unwrap_or_default(),
            )
        }
        Target::Unavailable => return Err("fonte indisponível".into()),
    };
    if job.cursor && !locked {
        elements.splice(
            0..0,
            super::cursor::elements(state, cursors, renderer, &job.target, origin, scale),
        );
    }
    let offset = job
        .region
        .map(|r| Point::from((-r.x, -r.y)).to_physical_precise_round(scale))
        .unwrap_or_default();
    let elements: Vec<_> = elements
        .into_iter()
        .map(|e| RelocateRenderElement::from_element(e, offset, Relocate::Relative))
        .collect();
    let size = (job.buffer.width() as i32, job.buffer.height() as i32);
    let mut framebuffer = renderer.bind(&mut job.buffer).map_err(|e| e.to_string())?;
    let mut damage = OutputDamageTracker::new(size, scale, Transform::Normal);
    let clear = match locked {
        true => [0.0, 0.0, 0.0, 1.0].into(),
        false => state.colors.background,
    };
    let result = damage
        .render_output(renderer, &mut framebuffer, 0, &elements, clear)
        .map_err(|e| e.to_string())?;
    Ok(result.sync)
}
