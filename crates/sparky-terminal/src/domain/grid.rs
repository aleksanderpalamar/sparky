const MIN_COLUMNS: usize = 2;
const MIN_LINES: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub cell: Cell,
    pub padding: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Half {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spot {
    pub line: usize,
    pub column: usize,
    pub half: Half,
}

impl Metrics {
    pub fn grid(&self, width: f32, height: f32) -> (usize, usize) {
        let fit = |space: f32, size: f32, minimum: usize| {
            (((space - 2.0 * self.padding) / size).floor().max(0.0) as usize).max(minimum)
        };
        (
            fit(width, self.cell.width, MIN_COLUMNS),
            fit(height, self.cell.height, MIN_LINES),
        )
    }

    pub fn spot(&self, x: f32, y: f32, grid: (usize, usize)) -> Spot {
        let (columns, lines) = grid;
        let across = ((x - self.padding) / self.cell.width).max(0.0);
        let down = ((y - self.padding) / self.cell.height).max(0.0);
        let column = (across.floor() as usize).min(columns.saturating_sub(1));
        let line = (down.floor() as usize).min(lines.saturating_sub(1));
        let beyond = across >= columns as f32;
        let half = match beyond || across.fract() >= 0.5 {
            true => Half::Right,
            false => Half::Left,
        };
        Spot { line, column, half }
    }
}
