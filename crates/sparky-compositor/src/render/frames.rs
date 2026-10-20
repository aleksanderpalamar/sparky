use std::collections::HashMap;

use smithay::backend::renderer::Color32F;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::element::solid::{SolidColorBuffer, SolidColorRenderElement};
use smithay::utils::{Logical, Point, Scale};
use sparky_tiling::{Rect, WindowId};

use crate::render::border;

#[derive(Default)]
pub struct Frames {
    buffers: HashMap<WindowId, [SolidColorBuffer; 4]>,
}

impl Frames {
    pub fn elements(
        &mut self,
        window: WindowId,
        frame: Rect,
        width: i32,
        color: Color32F,
        scale: Scale<f64>,
    ) -> Vec<SolidColorRenderElement> {
        if width <= 0 {
            return Vec::new();
        }
        let buffers = self.buffers.entry(window).or_default();
        border::edges(frame, width)
            .iter()
            .zip(buffers.iter_mut())
            .map(|(edge, buffer)| {
                buffer.update((edge.width, edge.height), color);
                let location: Point<i32, Logical> = (edge.x, edge.y).into();
                SolidColorRenderElement::from_buffer(
                    buffer,
                    location.to_physical_precise_round(scale),
                    scale,
                    1.0,
                    Kind::Unspecified,
                )
            })
            .collect()
    }

    pub fn forget(&mut self, window: WindowId) {
        self.buffers.remove(&window);
    }
}
