use smithay::backend::renderer::gles::Uniform;
use smithay::utils::{Logical, Physical, Rectangle, Size};

pub const GRAIN: f32 = 0.03;

pub struct Piece {
    pub clip: Vec<Rectangle<i32, Physical>>,
    pub rounding: Option<Vec<Uniform<'static>>>,
}

pub fn pieces(
    region: &[Rectangle<i32, Logical>],
    visible: Rectangle<f64, Logical>,
    dst: Rectangle<i32, Physical>,
    damage: &[Rectangle<i32, Physical>],
    whole: Size<f64, Logical>,
    radius: f32,
) -> Vec<Piece> {
    region
        .iter()
        .map(|rect| Piece {
            clip: clipped(std::slice::from_ref(rect), visible, dst, damage),
            rounding: rounding(Some(*rect), whole, radius),
        })
        .filter(|piece| !piece.clip.is_empty())
        .collect()
}

pub fn rounding(
    bounds: Option<Rectangle<i32, Logical>>,
    whole: Size<f64, Logical>,
    radius: f32,
) -> Option<Vec<Uniform<'static>>> {
    let bounds = bounds.filter(|_| whole.w > 0.0 && whole.h > 0.0)?;
    let rect = [
        (bounds.loc.x as f64 / whole.w) as f32,
        (bounds.loc.y as f64 / whole.h) as f32,
        (bounds.size.w as f64 / whole.w) as f32,
        (bounds.size.h as f64 / whole.h) as f32,
    ];
    let limit = bounds.size.w.min(bounds.size.h) as f32 / 2.0;
    Some(vec![
        Uniform::new("rect", rect),
        Uniform::new("size", [bounds.size.w as f32, bounds.size.h as f32]),
        Uniform::new("radius", radius.clamp(0.0, limit)),
        Uniform::new("grain", GRAIN),
    ])
}

fn clipped(
    region: &[Rectangle<i32, Logical>],
    visible: Rectangle<f64, Logical>,
    dst: Rectangle<i32, Physical>,
    damage: &[Rectangle<i32, Physical>],
) -> Vec<Rectangle<i32, Physical>> {
    let scale = dst.size.to_f64() / visible.size;
    region
        .iter()
        .filter_map(|rect| {
            let mut rect = rect.to_f64().intersection(visible)?;
            rect.loc -= visible.loc;
            Some(rect.to_physical_precise_round(scale))
        })
        .flat_map(|rect| {
            damage
                .iter()
                .filter_map(move |hurt| hurt.intersection(rect))
        })
        .collect()
}
