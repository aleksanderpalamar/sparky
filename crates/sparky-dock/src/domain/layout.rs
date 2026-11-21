pub const SLOT: f32 = 64.0;
pub const GLYPH: f32 = 40.0;
pub const GAP: f32 = 11.0;
pub const INSET_X: f32 = 15.0;
pub const INSET_Y: f32 = 12.0;
pub const MARK: f32 = 5.0;
pub const MARK_GAP: f32 = 5.0;
pub const DIVIDER: f32 = 1.0;
pub const DIVIDER_HEIGHT: f32 = 48.0;
pub const OUTLINE: f32 = 1.0;
pub const LIFT: i32 = 17;

pub fn width(pinned: usize) -> f32 {
    let slots = pinned as f32 + 1.0;
    (INSET_X + OUTLINE) * 2.0 + slots * SLOT + slots * GAP + DIVIDER
}

pub fn height() -> f32 {
    (INSET_Y + OUTLINE) * 2.0 + SLOT + MARK_GAP + MARK
}

pub fn size(pinned: usize) -> (u32, u32) {
    (width(pinned).ceil() as u32, height().ceil() as u32)
}

pub fn reserved() -> i32 {
    height().ceil() as i32
}
