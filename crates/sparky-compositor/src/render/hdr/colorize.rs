use smithay::backend::renderer::element::{Element, Id};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexProgram};

use super::converted::ConvertedSurface;
use super::surface_shader;
use crate::color_management::conversion::Conversion;
use crate::render::elements::SparkyElement;
use crate::state::Sparky;

smithay::backend::renderer::element::render_elements! {
    pub ColorElement<=GlesRenderer>;
    Plain=SparkyElement<GlesRenderer>,
    Converted=ConvertedSurface,
}

pub fn colorize(
    state: &Sparky,
    renderer: &mut GlesRenderer,
    elements: Vec<SparkyElement<GlesRenderer>>,
) -> Vec<ColorElement> {
    let conversions: Vec<(Id, Conversion)> = state
        .color_management
        .surfaces
        .conversions()
        .into_iter()
        .map(|(surface, conversion)| (Id::from_wayland_resource(&surface), conversion))
        .collect();
    let program = match conversions.is_empty() {
        true => None,
        false => surface_shader::program(renderer),
    };
    let Some(program) = program else {
        return elements.into_iter().map(ColorElement::Plain).collect();
    };
    elements
        .into_iter()
        .map(|element| convert(element, &conversions, &program))
        .collect()
}

fn convert(
    element: SparkyElement<GlesRenderer>,
    conversions: &[(Id, Conversion)],
    program: &GlesTexProgram,
) -> ColorElement {
    let SparkyElement::Surface(surface) = element else {
        return ColorElement::Plain(element);
    };
    match conversions.iter().find(|(id, _)| id == surface.id()) {
        Some((_, conversion)) => {
            ColorElement::Converted(ConvertedSurface::new(surface, program.clone(), conversion))
        }
        None => ColorElement::Plain(SparkyElement::Surface(surface)),
    }
}
