use sparky_ipc::{GlassArea, GlassRect};

use crate::domain::layout::{Geometry, LIFT, Layout};

pub struct Pose {
    pub scales: Vec<f32>,
    pub sink: f32,
    pub tucked: bool,
}

pub fn area(layout: &Layout, pose: &Pose, frame: Geometry, radius: i32) -> GlassArea {
    if pose.tucked {
        return GlassArea::Rects {
            rects: Vec::new(),
            radius,
        };
    }
    let width = layout.tray_width(&pose.scales).ceil() as i32;
    let height = layout.height().ceil() as i32;
    let tray = GlassRect {
        x: (frame.size.0 as i32 - width) / 2,
        y: frame.size.1 as i32 - LIFT as i32 - height + pose.sink.round() as i32,
        width,
        height,
    };
    GlassArea::Rects {
        rects: vec![tray],
        radius,
    }
}
