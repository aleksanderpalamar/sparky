use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::gles::{GlesError, GlesFrame, GlesTexture};
use smithay::backend::renderer::sync::SyncPoint;
use smithay::backend::renderer::{
    Bind, BlitFrame, Color32F, Frame, FrameContext, Offscreen, Renderer, Texture, TextureFilter,
};
use smithay::utils::{Buffer, Physical, Rectangle, Size, Transform};

pub fn copy_behind(
    frame: &mut GlesFrame<'_, '_>,
    src: Rectangle<f64, Buffer>,
    dst: Rectangle<i32, Physical>,
    into: &mut GlesTexture,
) -> Result<(), GlesError> {
    let transform = frame.transformation();
    let canvas = into.size().to_logical(1, Transform::Normal).to_physical(1);
    let output = frame.output_size();
    let mut guard = frame.renderer();
    let mut target = guard.as_mut().bind(into)?;
    let cleared = {
        let mut sub = guard
            .as_mut()
            .render(&mut target, canvas, Transform::Normal)?;
        sub.clear(Color32F::TRANSPARENT, &[Rectangle::from_size(canvas)])?;
        sub.finish()?
    };
    let destination = src
        .to_logical(1.0, Transform::Normal, &Size::default())
        .to_physical(1.0)
        .to_i32_round();
    if transform == Transform::Normal {
        drop(guard);
        frame.wait(&cleared)?;
        let sync = frame.blit_to(&mut target, dst, destination, TextureFilter::Linear)?;
        return frame.wait(&sync);
    }
    let rotated = transform.transform_rect_in(dst, &output);
    let size = rotated
        .to_logical(1)
        .to_buffer(1, Transform::Normal, &Size::default())
        .size;
    let mut scratch: GlesTexture = guard.as_mut().create_buffer(Fourcc::Abgr8888, size)?;
    let mut scratch_target = guard.as_mut().bind(&mut scratch)?;
    drop(guard);
    frame.wait(&cleared)?;
    let copied = frame.blit_to(
        &mut scratch_target,
        rotated,
        Rectangle::from_size(rotated.size),
        TextureFilter::Linear,
    )?;
    frame.wait(&copied)?;
    drop(scratch_target);
    let mut guard = frame.renderer();
    let mut sub = guard
        .as_mut()
        .render(&mut target, canvas, Transform::Normal)?;
    sub.wait(&copied)?;
    let whole = Rectangle::from_size(dst.size);
    Frame::render_texture_from_to(
        &mut sub,
        &scratch,
        Rectangle::from_size(size.to_f64()),
        destination,
        &[whole],
        &[whole],
        transform.invert(),
        1.0,
    )?;
    let sync: SyncPoint = sub.finish()?;
    drop(guard);
    frame.wait(&sync)
}
