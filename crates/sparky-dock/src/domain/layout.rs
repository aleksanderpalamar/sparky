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

impl Geometry {
    pub fn with_menu(self) -> Self {
        Self {
            size: (self.size.0.max(MENU_WIDTH as u32), self.size.1 + MENU_SPACE),
            ..self
        }
    }
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

    pub fn offset(&self, index: usize) -> f32 {
        INSET_X + OUTLINE + index as f32 * (self.slot + GAP)
    }

    pub fn geometry(&self, pinned: usize, visibility: Visibility, autohide: bool) -> Geometry {
        let (width, height) = self.size(pinned);
        let reach = height + LIFT as u32;
        match (visibility, autohide) {
            (Visibility::Hidden, _) => Geometry {
                size: (width, PEEK),
                zone: 0,
            },
            (Visibility::Shown, true) => Geometry {
                size: (width, reach),
                zone: 0,
            },
            (Visibility::Shown, false) => Geometry {
                size: (width, reach),
                zone: reach as i32,
            },
        }
    }
}
