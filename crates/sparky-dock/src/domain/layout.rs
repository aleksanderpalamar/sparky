pub const SLOT: f32 = 48.0;
pub const GLYPH: f32 = 28.0;
pub const GAP: f32 = 8.0;
pub const INSET: f32 = 8.0;
pub const MARK: f32 = 4.0;
pub const MARK_GAP: f32 = 3.0;
pub const DIVIDER: f32 = 1.0;
pub const LIFT: i32 = 8;

pub fn width(pinned: usize) -> f32 {
    let slots = pinned as f32 + 1.0;
    INSET * 2.0 + slots * SLOT + slots * GAP + DIVIDER
}

pub fn height() -> f32 {
    INSET * 2.0 + SLOT + MARK_GAP + MARK
}

pub fn size(pinned: usize) -> (u32, u32) {
    (width(pinned).ceil() as u32, height().ceil() as u32)
}

pub fn reserved() -> i32 {
    height().ceil() as i32
}
