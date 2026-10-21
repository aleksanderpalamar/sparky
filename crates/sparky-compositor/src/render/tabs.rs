use smithay::backend::renderer::Color32F;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::memory::MemoryRenderBufferRenderElement;
use smithay::backend::renderer::{ImportAll, ImportMem, Renderer, Texture};
use smithay::utils::{Logical, Point, Scale};
use sparky_tiling::Rect;
use sparky_tiling::workspace_groups::Tabs;

use crate::render::elements::SparkyElement;
use crate::state::Sparky;
use crate::tabs::lettering::Emphasis;
use crate::tabs::paint::Part;
use crate::tabs::slots::{indicator, slots};

const ACTIVE_FILL: Color32F = Color32F::new(0.0, 0.0, 0.0, 0x40 as f32 / 255.0);
const INACTIVE_FILL: Color32F = Color32F::new(0.0, 0.0, 0.0, 0x20 as f32 / 255.0);

pub fn tab_elements<R>(state: &mut Sparky, renderer: &mut R, tabs: &Tabs, bar: Rect, scale: Scale<f64>) -> Vec<SparkyElement<R>>
where
    R: Renderer + ImportAll + ImportMem,
    R::TextureId: Clone + Texture + Send + 'static,
{
    let mut elements = Vec::new();
    for (index, (slot, member)) in slots(bar, tabs.members.len()).into_iter().zip(&tabs.members).enumerate() {
        let emphasis = Emphasis::of(index == tabs.active);
        let title = state.title(*member);
        if let Some(label) = state.lettering.label(&title, (slot.width, slot.height - 1), emphasis) {
            let location: Point<i32, Logical> = (slot.x, slot.y).into();
            let placed = MemoryRenderBufferRenderElement::from_buffer(
                renderer,
                location.to_f64().to_physical(scale),
                label,
                None,
                None,
                None,
                Kind::Unspecified,
            );
            if let Ok(element) = placed {
                elements.push(SparkyElement::Memory(element));
            }
        }
        let (line, fill) = match emphasis {
            Emphasis::Active => (state.colors.for_focus(tabs.focused), ACTIVE_FILL),
            Emphasis::Inactive => (state.colors.inactive, INACTIVE_FILL),
        };
        elements.push(SparkyElement::Solid(state.tab_paint.element((*member, Part::Indicator), indicator(slot), line, scale)));
        elements.push(SparkyElement::Solid(state.tab_paint.element((*member, Part::Background), slot, fill, scale)));
    }
    elements
}
