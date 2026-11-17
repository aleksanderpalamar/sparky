use std::path::{Path, PathBuf};

use crate::colors_file;
use crate::rgb::Rgb;

const ANSI_KEYS: [&str; 16] = [
    "background",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "foreground",
    "muted",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_foreground",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalPalette {
    pub ansi: [Rgb; 16],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    pub cursor_text: Rgb,
    pub selection_foreground: Rgb,
    pub selection_background: Rgb,
}

impl TerminalPalette {
    pub const TOKYO_NIGHT: Self = Self {
        ansi: [
            Rgb::new(0x1a, 0x1b, 0x26),
            Rgb::new(0xf7, 0x76, 0x8e),
            Rgb::new(0x9e, 0xce, 0x6a),
            Rgb::new(0xe0, 0xaf, 0x68),
            Rgb::new(0x7a, 0xa2, 0xf7),
            Rgb::new(0xad, 0x8e, 0xe6),
            Rgb::new(0x44, 0x9d, 0xab),
            Rgb::new(0xa9, 0xb1, 0xd6),
            Rgb::new(0x41, 0x48, 0x68),
            Rgb::new(0xff, 0x7a, 0x93),
            Rgb::new(0xb9, 0xf2, 0x7c),
            Rgb::new(0xff, 0x9e, 0x64),
            Rgb::new(0x7d, 0xa6, 0xff),
            Rgb::new(0xbb, 0x9a, 0xf7),
            Rgb::new(0x0d, 0xb9, 0xd7),
            Rgb::new(0xc0, 0xca, 0xf5),
        ],
        foreground: Rgb::new(0xa9, 0xb1, 0xd6),
        background: Rgb::new(0x1a, 0x1b, 0x26),
        cursor: Rgb::new(0xc0, 0xca, 0xf5),
        cursor_text: Rgb::new(0x1a, 0x1b, 0x26),
        selection_foreground: Rgb::new(0xc0, 0xca, 0xf5),
        selection_background: Rgb::new(0x29, 0x2e, 0x42),
    };

    pub const SPARKY: Self = Self {
        ansi: [
            Rgb::new(0x30, 0x0a, 0x24),
            Rgb::new(0xc0, 0x1c, 0x28),
            Rgb::new(0x26, 0xa2, 0x69),
            Rgb::new(0xa2, 0x73, 0x4c),
            Rgb::new(0x12, 0x48, 0x8b),
            Rgb::new(0xa3, 0x47, 0xba),
            Rgb::new(0x2a, 0xa1, 0xb3),
            Rgb::new(0xff, 0xff, 0xff),
            Rgb::new(0x5e, 0x5c, 0x64),
            Rgb::new(0xf6, 0x61, 0x51),
            Rgb::new(0x33, 0xda, 0x7a),
            Rgb::new(0xe9, 0xad, 0x0c),
            Rgb::new(0x2a, 0x7b, 0xde),
            Rgb::new(0xc0, 0x61, 0xcb),
            Rgb::new(0x33, 0xc7, 0xde),
            Rgb::new(0xff, 0xff, 0xff),
        ],
        foreground: Rgb::new(0xff, 0xff, 0xff),
        background: Rgb::new(0x30, 0x0a, 0x24),
        cursor: Rgb::new(0xff, 0xff, 0xff),
        cursor_text: Rgb::new(0x30, 0x0a, 0x24),
        selection_foreground: Rgb::new(0xff, 0xff, 0xff),
        selection_background: Rgb::new(0x77, 0x21, 0x6f),
    };

    pub fn load(read: impl Fn(&Path) -> Option<String>, candidates: &[PathBuf]) -> Self {
        candidates
            .iter()
            .find_map(|path| read(path))
            .map(|contents| Self::from_colors(&contents))
            .unwrap_or(Self::SPARKY)
    }

    pub fn from_colors(contents: &str) -> Self {
        let fallback = Self::TOKYO_NIGHT;
        let color = |key: &str, default: Rgb| {
            terminal_value(contents, key)
                .and_then(Rgb::parse)
                .unwrap_or(default)
        };
        let mut ansi = fallback.ansi;
        for (slot, key) in ansi.iter_mut().zip(ANSI_KEYS) {
            *slot = color(key, *slot);
        }
        let background = color("background", fallback.background);
        let bright = color("bright_foreground", fallback.cursor);
        Self {
            ansi,
            foreground: color("foreground", fallback.foreground),
            background,
            cursor: bright,
            cursor_text: background,
            selection_foreground: bright,
            selection_background: color("selection", fallback.selection_background),
        }
    }
}

fn terminal_value<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    colors_file::value(contents, &format!("terminal_{key}"))
        .or_else(|| colors_file::value(contents, key))
}
