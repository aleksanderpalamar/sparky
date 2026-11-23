use super::layout::{Geometry, Layout, MENU_SPACE, PEEK};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Tucked,
    Resting,
    Lifted,
    Menu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn region(layout: &Layout, pinned: usize, frame: Geometry, stage: Stage) -> Rect {
    let (surface, bottom) = (frame.size.0 as i32, frame.size.1 as i32);
    let tray = layout.size(pinned).0 as i32;
    let centered = |width: i32, height: i32| Rect {
        x: (surface - width) / 2,
        y: bottom - height,
        width,
        height,
    };
    let rest = layout.rest() as i32;
    match stage {
        Stage::Tucked => centered(tray, PEEK as i32),
        Stage::Resting => centered(tray, rest),
        Stage::Lifted => centered(surface, rest + layout.rise() as i32),
        Stage::Menu => centered(surface, rest + MENU_SPACE as i32),
    }
}
