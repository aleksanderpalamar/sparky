use sparky_ipc::{GlassArea, GlassRect};

use crate::domain::layout::{Geometry, LIFT, Layout};

pub fn area(
    layout: &Layout,
    pinned: usize,
    frame: Geometry,
    tucked: bool,
    radius: i32,
) -> GlassArea {
    if tucked {
        return GlassArea::Rects {
            rects: Vec::new(),
            radius,
        };
    }
    let (width, height) = layout.size(pinned);
    let tray = GlassRect {
        x: (frame.size.0 as i32 - width as i32) / 2,
        y: frame.size.1 as i32 - LIFT as i32 - height as i32,
        width: width as i32,
        height: height as i32,
    };
    GlassArea::Rects {
        rects: vec![tray],
        radius,
    }
}
