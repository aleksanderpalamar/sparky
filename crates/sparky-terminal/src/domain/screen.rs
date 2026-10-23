use sparky_theme::Rgb;

use crate::domain::runs::Run;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorSpot {
    pub line: usize,
    pub column: usize,
    pub character: char,
    pub color: Rgb,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Screen {
    pub rows: Vec<Vec<Run>>,
    pub cursor: Option<CursorSpot>,
}
