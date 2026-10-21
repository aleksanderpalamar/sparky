use std::collections::HashMap;

use smithay::backend::renderer::Color32F;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::solid::{SolidColorBuffer, SolidColorRenderElement};
use smithay::utils::{Logical, Point, Scale};
use sparky_tiling::{Rect, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    Background,
    Indicator,
}

#[derive(Default)]
pub struct TabPaint {
    buffers: HashMap<(WindowId, Part), SolidColorBuffer>,
}

impl TabPaint {
    pub fn element(&mut self, key: (WindowId, Part), area: Rect, color: Color32F, scale: Scale<f64>) -> SolidColorRenderElement {
        let buffer = self.buffers.entry(key).or_default();
        buffer.update((area.width, area.height), color);
        let location: Point<i32, Logical> = (area.x, area.y).into();
        SolidColorRenderElement::from_buffer(buffer, location.to_physical_precise_round(scale), scale, 1.0, Kind::Unspecified)
    }

    pub fn forget(&mut self, window: WindowId) {
        self.buffers.retain(|(owner, _), _| *owner != window);
    }
}
