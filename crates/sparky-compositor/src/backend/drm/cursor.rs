use std::collections::HashMap;

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::input::pointer::{CursorIcon, CursorImageStatus};
use smithay::utils::{Logical, Point, Transform};

use crate::cursor::{CursorPixels, names_for};

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

pub struct Cursors {
    default: CursorImage,
    named: HashMap<CursorIcon, Option<CursorImage>>,
}

impl Cursors {
    pub fn load() -> Self {
        Self {
            default: CursorImage::from_pixels(&CursorPixels::load()),
            named: HashMap::new(),
        }
    }

    pub fn image(&mut self, status: &CursorImageStatus) -> &CursorImage {
        let CursorImageStatus::Named(icon) = status else {
            return &self.default;
        };
        self.named
            .entry(*icon)
            .or_insert_with(|| CursorPixels::named(&names_for(*icon)).map(|pixels| CursorImage::from_pixels(&pixels)))
            .as_ref()
            .unwrap_or(&self.default)
    }
}
