use crate::geometry::Rect;

pub const MINIMUM: (i32, i32) = (120, 80);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Horizontal {
    Left,
    Right,
    Neither,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    Top,
    Bottom,
    Neither,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edges {
    pub horizontal: Horizontal,
    pub vertical: Vertical,
}

impl Edges {
    pub fn nearest(frame: Rect, x: i32, y: i32) -> Self {
        let horizontal = match x < frame.x + frame.width / 2 {
            true => Horizontal::Left,
            false => Horizontal::Right,
        };
        let vertical = match y < frame.y + frame.height / 2 {
            true => Vertical::Top,
            false => Vertical::Bottom,
        };
        Self { horizontal, vertical }
    }

    pub fn growth(self, delta: (i32, i32)) -> (i32, i32) {
        let width = match self.horizontal {
            Horizontal::Left => -delta.0,
            Horizontal::Right => delta.0,
            Horizontal::Neither => 0,
        };
        let height = match self.vertical {
            Vertical::Top => -delta.1,
            Vertical::Bottom => delta.1,
            Vertical::Neither => 0,
        };
        (width, height)
    }
}

pub fn moved(frame: Rect, delta: (i32, i32)) -> Rect {
    Rect::new(frame.x + delta.0, frame.y + delta.1, frame.width, frame.height)
}

pub fn resized(frame: Rect, edges: Edges, delta: (i32, i32)) -> Rect {
    let (grow_width, grow_height) = edges.growth(delta);
    let width = (frame.width + grow_width).max(MINIMUM.0);
    let height = (frame.height + grow_height).max(MINIMUM.1);
    let x = match edges.horizontal {
        Horizontal::Left => frame.right() - width,
        Horizontal::Right | Horizontal::Neither => frame.x,
    };
    let y = match edges.vertical {
        Vertical::Top => frame.bottom() - height,
        Vertical::Bottom | Vertical::Neither => frame.y,
    };
    Rect::new(x, y, width, height)
}
