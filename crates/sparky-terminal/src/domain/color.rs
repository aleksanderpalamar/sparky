use alacritty_terminal::vte::ansi::{Color, NamedColor};
use sparky_theme::{Rgb, TerminalPalette};

const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];
const CUBE_START: usize = 16;
const GRAY_START: usize = 232;
const FOREGROUND: usize = 256;
const BACKGROUND: usize = 257;
const CURSOR: usize = 258;

pub fn resolve(
    color: Color,
    palette: &TerminalPalette,
    overrides: &dyn Fn(usize) -> Option<Rgb>,
) -> Rgb {
    match color {
        Color::Spec(spec) => Rgb::new(spec.r, spec.g, spec.b),
        Color::Indexed(index) => indexed(usize::from(index), palette, overrides),
        Color::Named(named) => {
            overrides(named as usize).unwrap_or_else(|| named_color(named, palette))
        }
    }
}

pub fn by_index(index: usize, palette: &TerminalPalette) -> Rgb {
    match (index, u8::try_from(index)) {
        (_, Ok(indexed)) => resolve(Color::Indexed(indexed), palette, &|_| None),
        (BACKGROUND, _) => palette.background,
        (CURSOR, _) => palette.cursor,
        (FOREGROUND, _) | (_, Err(_)) => palette.foreground,
    }
}

pub fn dim(color: Rgb) -> Rgb {
    let channel = |value: u8| (u16::from(value) * 2 / 3) as u8;
    Rgb::new(
        channel(color.red),
        channel(color.green),
        channel(color.blue),
    )
}

fn indexed(
    index: usize,
    palette: &TerminalPalette,
    overrides: &dyn Fn(usize) -> Option<Rgb>,
) -> Rgb {
    if let Some(color) = overrides(index) {
        return color;
    }
    match index {
        0..CUBE_START => palette.ansi[index],
        CUBE_START..GRAY_START => cube(index - CUBE_START),
        _ => gray(index - GRAY_START),
    }
}

fn cube(offset: usize) -> Rgb {
    Rgb::new(CUBE[offset / 36], CUBE[(offset % 36) / 6], CUBE[offset % 6])
}

fn gray(offset: usize) -> Rgb {
    let level = (8 + offset * 10) as u8;
    Rgb::new(level, level, level)
}

fn named_color(named: NamedColor, palette: &TerminalPalette) -> Rgb {
    match named {
        NamedColor::Foreground => palette.foreground,
        NamedColor::Background => palette.background,
        NamedColor::Cursor => palette.cursor,
        NamedColor::BrightForeground => palette.ansi[15],
        NamedColor::DimForeground => dim(palette.foreground),
        NamedColor::DimBlack => dim(palette.ansi[0]),
        NamedColor::DimRed => dim(palette.ansi[1]),
        NamedColor::DimGreen => dim(palette.ansi[2]),
        NamedColor::DimYellow => dim(palette.ansi[3]),
        NamedColor::DimBlue => dim(palette.ansi[4]),
        NamedColor::DimMagenta => dim(palette.ansi[5]),
        NamedColor::DimCyan => dim(palette.ansi[6]),
        NamedColor::DimWhite => dim(palette.ansi[7]),
        normal => palette.ansi[(normal as usize).min(CUBE_START - 1)],
    }
}
