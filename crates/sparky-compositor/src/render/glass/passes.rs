use smithay::backend::renderer::gles::{GlesError, GlesRenderer, GlesTexture, Uniform, ffi};
use smithay::backend::renderer::{Bind, Color32F, Frame, Renderer, Texture};
use smithay::utils::{Buffer, Physical, Rectangle, Size, Transform};

use super::shaders::Shaders;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Down,
    Up,
}

pub fn blur(
    renderer: &mut GlesRenderer,
    shaders: &Shaders,
    textures: &mut [&mut GlesTexture; 2],
    offset: f32,
    passes: usize,
) -> Result<(), GlesError> {
    for level in 0..passes {
        pass(renderer, shaders, textures, offset, level, Direction::Down)?;
        textures.swap(0, 1);
    }
    for level in (0..passes).rev() {
        pass(renderer, shaders, textures, offset, level, Direction::Up)?;
        textures.swap(0, 1);
    }
    Ok(())
}

fn pass(
    renderer: &mut GlesRenderer,
    shaders: &Shaders,
    textures: &mut [&mut GlesTexture; 2],
    offset: f32,
    level: usize,
    direction: Direction,
) -> Result<(), GlesError> {
    let full = textures[0].size();
    let (from, to, program, step) = match direction {
        Direction::Down => (
            full.downscale(1 << level),
            full.downscale(1 << (level + 1)),
            &shaders.down,
            1 << level,
        ),
        Direction::Up => (
            full.downscale(1 << (level + 1)),
            full.downscale(1 << level),
            &shaders.up,
            1 << (level + 1),
        ),
    };
    let [source, target] = textures;
    let mut framebuffer = renderer.bind(*target)?;
    let canvas = physical(full);
    let mut frame = renderer.render(&mut framebuffer, canvas, Transform::Normal)?;
    frame.clear(
        Color32F::new(0.0, 0.0, 0.0, 0.0),
        &[Rectangle::from_size(canvas)],
    )?;
    clamp(&mut frame, ffi::CLAMP_TO_EDGE)?;
    let half_pixel = [0.5 / from.w.max(1) as f32, 0.5 / from.h.max(1) as f32];
    let drawn = physical(to);
    frame.render_texture_from_to(
        source,
        Rectangle::from_size(from.to_f64()),
        Rectangle::from_size(drawn),
        &[Rectangle::from_size(drawn)],
        &[Rectangle::from_size(drawn)],
        Transform::Normal,
        1.0,
        Some(program),
        &[
            Uniform::new("half_pixel", half_pixel),
            Uniform::new("offset", offset / step as f32),
        ],
    )?;
    clamp(&mut frame, ffi::REPEAT)?;
    let sync = frame.finish()?;
    drop(framebuffer);
    renderer.wait(&sync)
}

fn clamp(
    frame: &mut smithay::backend::renderer::gles::GlesFrame<'_, '_>,
    mode: u32,
) -> Result<(), GlesError> {
    frame.with_context(|gl| unsafe {
        gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_S, mode as i32);
        gl.TexParameteri(ffi::TEXTURE_2D, ffi::TEXTURE_WRAP_T, mode as i32);
    })
}

fn physical(size: Size<i32, Buffer>) -> Size<i32, Physical> {
    size.to_logical(1, Transform::Normal).to_physical(1)
}
