use smithay::backend::renderer::element::{Element, Id};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexProgram};

use super::converted::ConvertedSurface;
use super::{identity, surface_shader};
use crate::color_management::conversion::Conversion;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;

smithay::backend::renderer::element::render_elements! {
    pub ColorElement<=GlesRenderer>;
    Plain=SparkyElement<GlesRenderer>,
    Converted=ConvertedSurface,
}

struct Target {
    surface: Id,
    element: Id,
    conversion: Conversion,
}

pub fn colorize(
    state: &Sparky,
    renderer: &mut GlesRenderer,
    elements: Vec<SparkyElement<GlesRenderer>>,
) -> Vec<ColorElement> {
    let targets: Vec<Target> = state
        .color_management
        .surfaces
        .conversions()
        .into_iter()
        .map(|(surface, conversion)| Target {
            surface: Id::from_wayland_resource(&surface),
            element: identity::element_id(&surface, conversion),
            conversion,
        })
        .collect();
    let program = match targets.is_empty() {
        true => None,
        false => surface_shader::program(renderer),
    };
    let Some(program) = program else {
        return elements.into_iter().map(ColorElement::Plain).collect();
    };
    elements
        .into_iter()
        .map(|element| convert(element, &targets, &program))
        .collect()
}

fn convert(
    element: SparkyElement<GlesRenderer>,
    targets: &[Target],
    program: &GlesTexProgram,
) -> ColorElement {
    let SparkyElement::Surface(surface) = element else {
        return ColorElement::Plain(element);
    };
    match targets
        .iter()
        .find(|target| &target.surface == surface.id())
    {
        Some(target) => ColorElement::Converted(ConvertedSurface::new(
            surface,
            target.element.clone(),
            program.clone(),
            &target.conversion,
        )),
        None => ColorElement::Plain(SparkyElement::Surface(surface)),
    }
}
