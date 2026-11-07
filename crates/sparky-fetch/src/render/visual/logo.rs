use crate::render::visual::ansi;
use crate::render::visual::segment::{Line, Segment, Tone};

const MARK: [&str; 11] = [
    "             ▄▄▄▄▄▄▄▄▄",
    "   +       ▄████████▀",
    "         ▄███████▀",
    "       ▄███████▀      *",
    "     ▄█████████████▄",
    "      ▀▀▀▀▀▀██████▀",
    "  *       ▄█████▀",
    "        ▄████▀      +",
    "       ▄███▀",
    "      ▄█▀",
    "     ▀",
];

pub(crate) fn lines() -> Vec<Line> {
    MARK.iter()
        .map(|row| Line::new(vec![Segment::new(*row, Tone::Logo)]))
        .collect()
}

pub(crate) fn width() -> usize {
    MARK.iter().map(|row| ansi::width(row)).max().unwrap_or(0)
}
