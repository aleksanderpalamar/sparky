use smithay::utils::{Logical, Rectangle};
use sparky_ipc::GlassArea;

pub fn rects(area: &GlassArea, surface: Rectangle<i32, Logical>) -> Vec<Rectangle<i32, Logical>> {
    let inside =
        |rect: Rectangle<i32, Logical>| rect.intersection(surface).filter(|rect| !rect.is_empty());
    match area {
        GlassArea::Whole => vec![surface],
        GlassArea::Centered { width, height } => {
            let size = (*width.min(&surface.size.w), *height.min(&surface.size.h));
            let x = surface.loc.x + (surface.size.w - size.0) / 2;
            let y = surface.loc.y + (surface.size.h - size.1) / 2;
            inside(Rectangle::new((x, y).into(), size.into()))
                .into_iter()
                .collect()
        }
        GlassArea::Rects { rects, .. } => rects
            .iter()
            .map(|rect| {
                Rectangle::new(
                    (surface.loc.x + rect.x, surface.loc.y + rect.y).into(),
                    (rect.width, rect.height).into(),
                )
            })
            .filter_map(inside)
            .collect(),
    }
}

pub fn radius(area: &GlassArea) -> i32 {
    match area {
        GlassArea::Rects { radius, .. } => (*radius).max(0),
        _ => 0,
    }
}
