use super::types::Target;
use crate::backend::drm::cursor::Cursors;
use crate::render::cursor::{Pointer, cursor_elements};
use crate::render::elements::SparkyElement;
use crate::state::Sparky;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::{Logical, Point, Scale};

pub fn elements(
    state: &Sparky,
    cursors: &mut Cursors,
    renderer: &mut GlesRenderer,
    target: &Target,
    origin: Point<i32, Logical>,
    scale: Scale<f64>,
) -> Vec<SparkyElement<GlesRenderer>> {
    let Some(pointer) = state.seat.get_pointer() else {
        return Vec::new();
    };
    let location = pointer.current_location();
    if !inside(state, target, location) {
        return Vec::new();
    }
    let image = cursors.image(&state.cursor_status);
    let pointer = Pointer {
        status: &state.cursor_status,
        image: &image.buffer,
        hotspot: image.hotspot,
        location: location - origin.to_f64(),
    };
    cursor_elements(renderer, &pointer, scale)
}

fn inside(state: &Sparky, target: &Target, location: Point<f64, Logical>) -> bool {
    match target {
        Target::Output(weak) => weak
            .upgrade()
            .and_then(|out| state.space.output_geometry(&out))
            .is_some_and(|g| g.to_f64().contains(location)),
        Target::Window(id) => state
            .space
            .element_under(location)
            .is_some_and(|(window, _)| state.windows.get(*id) == Some(window)),
        Target::Unavailable => false,
    }
}
