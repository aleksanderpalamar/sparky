use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::renderer::{Bind, ExportMem, Offscreen, TextureMapping};
use smithay::output::Output;
use smithay::utils::{Rectangle, Transform};
use sparky_tiling::Rect;

use crate::capture::area::CaptureError;
use crate::capture::pixels::flip_rows;
use crate::render::output::output_elements;
use crate::state::Sparky;

const BYTES_PER_PIXEL: usize = 4;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn read(state: &mut Sparky, renderer: &mut GlesRenderer, output: &Output, region: Rect) -> Result<Image, CaptureError> {
    let mode = output.current_mode().ok_or(CaptureError::NoOutput)?;
    let scale = output.current_scale().fractional_scale();
    let elements = output_elements(state, renderer, output);
    let mut texture: GlesTexture = renderer
        .create_buffer(Fourcc::Abgr8888, (mode.size.w, mode.size.h).into())
        .map_err(failure)?;
    let mut framebuffer = renderer.bind(&mut texture).map_err(failure)?;
    let mut damage = OutputDamageTracker::new(mode.size, scale, Transform::Normal);
    damage
        .render_output(renderer, &mut framebuffer, 0, &elements, state.colors.background)
        .map_err(|error| CaptureError::Render(format!("{error:?}")))?;
    let physical = scaled(region, scale);
    let area = Rectangle::new((physical.x, physical.y).into(), (physical.width, physical.height).into());
    let mapping = renderer
        .copy_framebuffer(&framebuffer, area, Fourcc::Abgr8888)
        .map_err(failure)?;
    let bytes = renderer.map_texture(&mapping).map_err(failure)?;
    let stride = physical.width as usize * BYTES_PER_PIXEL;
    let rgba = match mapping.flipped() {
        true => bytes.to_vec(),
        false => flip_rows(bytes, stride),
    };
    Ok(Image {
        width: physical.width as u32,
        height: physical.height as u32,
        rgba,
    })
}

pub fn scaled(region: Rect, scale: f64) -> Rect {
    let value = |number: i32| (number as f64 * scale).round() as i32;
    Rect::new(value(region.x), value(region.y), value(region.width), value(region.height))
}

fn failure(error: impl std::fmt::Display) -> CaptureError {
    CaptureError::Render(error.to_string())
}
