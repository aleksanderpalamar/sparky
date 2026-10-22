use std::ops::Range;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub columns: usize,
    pub rows: usize,
}

impl Layout {
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows: rows.max(1),
        }
    }

    pub fn capacity(self) -> usize {
        self.columns.saturating_mul(self.rows)
    }

    pub fn row_count(self, len: usize) -> usize {
        len.div_ceil(self.columns)
    }

    pub fn range(self, top: usize, len: usize) -> Range<usize> {
        top.saturating_mul(self.columns).min(len)..top.saturating_add(self.rows).saturating_mul(self.columns).min(len)
    }

    pub fn neighbor(self, cursor: usize, len: usize, direction: Direction) -> usize {
        let last = len.saturating_sub(1);
        let cursor = cursor.min(last);
        let column = cursor % self.columns;
        match direction {
            Direction::Up if cursor < self.columns => cursor,
            Direction::Down if cursor / self.columns == last / self.columns => cursor,
            Direction::Up => cursor.saturating_sub(self.columns),
            Direction::Down => cursor.saturating_add(self.columns).min(last),
            Direction::Left => cursor.saturating_sub(usize::from(column > 0)),
            Direction::Right => cursor.saturating_add(usize::from(column + 1 < self.columns)).min(last),
        }
    }
}
