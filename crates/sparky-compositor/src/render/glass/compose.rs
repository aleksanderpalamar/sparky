use std::sync::Mutex;

use smithay::backend::renderer::element::{Element, Id};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::output::Output;
use smithay::wayland::compositor::with_states;

use super::element::GlassElement;
use super::panes::{Pane, panes};
use super::shaders::Shaders;
use super::state::GlassState;
use crate::render::elements::SparkyElement;
use crate::render::hdr::colorize::{ColorElement, colorize};
use crate::state::Sparky;

pub fn compose(
    state: &Sparky,
    renderer: &mut GlesRenderer,
    output: &Output,
    elements: Vec<SparkyElement<GlesRenderer>>,
) -> Vec<ColorElement> {
    let mut composed = colorize(state, renderer, elements);
    let panes = panes(state, output);
    if panes.is_empty() {
        return composed;
    }
    let shaders = match Shaders::get(renderer) {
        Ok(shaders) => shaders,
        Err(error) => {
            tracing::warn!("frosted glass sem shaders: {error}");
            return composed;
        }
    };
    let scale = output.current_scale().fractional_scale();
    for pane in panes {
        let id = Id::from_wayland_resource(&pane.surface);
        let Some(index) = composed.iter().position(|element| element.id() == &id) else {
            continue;
        };
        if let Some(glass) = glass(&pane, shaders.clone(), scale) {
            composed.insert(index + 1, ColorElement::Glass(glass));
        }
    }
    composed
}

fn glass(pane: &Pane, shaders: Shaders, scale: f64) -> Option<GlassElement> {
    with_states(&pane.surface, |states| {
        let slot = states
            .data_map
            .get_or_insert_threadsafe(|| Mutex::new(GlassState::default()));
        let mut state = slot.lock().ok()?;
        GlassElement::new(shaders, &mut state, pane.region.clone(), scale, pane.radius)
    })
}
