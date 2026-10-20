use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::utils::{Logical, Point, Transform};

use crate::cursor::CursorPixels;

#[derive(Debug, Clone)]
pub struct CursorImage {
    pub buffer: MemoryRenderBuffer,
    pub hotspot: Point<i32, Logical>,
}

impl CursorImage {
    pub fn from_pixels(pixels: &CursorPixels) -> Self {
        Self {
            buffer: MemoryRenderBuffer::from_slice(
                &pixels.rgba,
                Fourcc::Abgr8888,
                (pixels.width as i32, pixels.height as i32),
                1,
                Transform::Normal,
                None,
            ),
            hotspot: (pixels.hotspot.0 as i32, pixels.hotspot.1 as i32).into(),
        }
    }
}
