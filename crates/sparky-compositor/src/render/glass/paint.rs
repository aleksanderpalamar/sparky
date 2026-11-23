use smithay::backend::renderer::Frame;
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesTexture};
use smithay::utils::{Buffer, Physical, Rectangle, Transform};

use super::clip::Piece;
use super::shaders::Shaders;

pub fn paint(
    frame: &mut GlesFrame<'_, '_>,
    texture: &GlesTexture,
    shaders: &Shaders,
    piece: Piece,
    src: Rectangle<f64, Buffer>,
    dst: Rectangle<i32, Physical>,
    opaque: &[Rectangle<i32, Physical>],
) -> Result<(), GlesError> {
    let rounded = piece.rounding.is_some();
    if let Some(uniforms) = piece.rounding {
        frame.override_default_tex_program(shaders.clip.clone(), uniforms);
    }
    let drawn = Frame::render_texture_from_to(
        frame,
        texture,
        src,
        dst,
        &piece.clip,
        opaque,
        Transform::Normal,
        1.0,
    );
    if rounded {
        frame.clear_tex_program_override();
    }
    drawn
}
