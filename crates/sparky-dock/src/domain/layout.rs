use super::magnify::PEAK;

pub const GAP: f32 = 11.0;
pub const INSET_X: f32 = 15.0;
pub const INSET_Y: f32 = 12.0;
pub const MARK: f32 = 5.0;
pub const MARK_GAP: f32 = 5.0;
pub const DIVIDER: f32 = 1.0;
pub const OUTLINE: f32 = 1.0;
pub const PEEK: u32 = 4;
pub const MENU_WIDTH: f32 = 200.0;
pub const MENU_SPACE: u32 = 92;
pub const LIFT: f32 = 17.0;
const SPREAD: f32 = 3.0;
const GLYPH_RATIO: f32 = 40.0 / 64.0;
const DIVIDER_RATIO: f32 = 48.0 / 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Shown,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Geometry {
    pub size: (u32, u32),
    pub zone: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub slot: f32,
}

impl Layout {
    pub fn new(slot: i32) -> Self {
        Self { slot: slot as f32 }
    }

    pub fn glyph(&self) -> f32 {
        self.slot * GLYPH_RATIO
    }

    pub fn divider(&self) -> f32 {
        self.slot * DIVIDER_RATIO
    }

    pub fn gap(&self) -> f32 {
        GAP
    }

    pub fn width(&self, pinned: usize) -> f32 {
        let slots = pinned as f32 + 1.0;
        (INSET_X + OUTLINE) * 2.0 + slots * self.slot + slots * GAP + DIVIDER
    }

    pub fn height(&self) -> f32 {
        (INSET_Y + OUTLINE) * 2.0 + self.slot + MARK_GAP + MARK
    }

    pub fn size(&self, pinned: usize) -> (u32, u32) {
        (
            self.width(pinned).ceil() as u32,
            self.height().ceil() as u32,
        )
    }

    pub fn tray_width(&self, scales: &[f32]) -> f32 {
        let slots: f32 = scales.iter().map(|scale| scale * self.slot).sum();
        (INSET_X + OUTLINE) * 2.0 + slots + scales.len() as f32 * GAP + DIVIDER
    }

    pub fn centers(&self, pinned: usize, surface: f32) -> Vec<f32> {
        let left = (surface - self.width(pinned)) / 2.0;
        let pitch = self.slot + GAP;
        let start = left + INSET_X + OUTLINE + self.slot / 2.0;
        (0..pinned)
            .map(|index| start + index as f32 * pitch)
            .chain(std::iter::once(
                start + pinned as f32 * pitch + DIVIDER + GAP,
            ))
            .collect()
    }

    pub fn offset(&self, index: usize) -> f32 {
        INSET_X + OUTLINE + index as f32 * (self.slot + GAP)
    }

    pub fn rise(&self) -> u32 {
        (self.slot * (PEAK - 1.0)).ceil() as u32
    }

    pub fn rest(&self) -> u32 {
        self.height().ceil() as u32 + LIFT as u32
    }

    pub fn frame(&self, pinned: usize, autohide: bool) -> Geometry {
        let width = self.size(pinned).0 + (self.rise() as f32 * SPREAD).ceil() as u32;
        let height = self.rest() + self.rise() + MENU_SPACE;
        let zone = match autohide {
            true => 0,
            false => self.rest() as i32,
        };
        Geometry {
            size: (width.max(MENU_WIDTH as u32), height),
            zone,
        }
    }
}
