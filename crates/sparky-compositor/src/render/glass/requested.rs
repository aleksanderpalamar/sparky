use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Size};
use smithay::wayland::background_effect::BackgroundEffectSurfaceCachedState;
use smithay::wayland::compositor::{RectangleKind, RegionAttributes, with_states};

pub fn requested(surface: &WlSurface) -> Option<RegionAttributes> {
    with_states(surface, |states| {
        states
            .cached_state
            .get::<BackgroundEffectSurfaceCachedState>()
            .current()
            .blur_region
            .clone()
    })
}

pub fn rects(
    region: &RegionAttributes,
    origin: Point<i32, Logical>,
    size: Size<i32, Logical>,
) -> Vec<Rectangle<i32, Logical>> {
    let bounds = Rectangle::from_size(size);
    region
        .rects
        .iter()
        .fold(Vec::new(), |kept, (kind, rect)| match kind {
            RectangleKind::Add => {
                let mut kept = kept;
                kept.extend(rect.intersection(bounds));
                kept
            }
            RectangleKind::Subtract => Rectangle::subtract_rects_many_in_place(kept, [*rect]),
        })
        .into_iter()
        .filter(|rect| !rect.is_empty())
        .map(|rect| Rectangle::new(rect.loc + origin, rect.size))
        .collect()
}
