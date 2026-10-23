use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::Color;
use sparky_theme::{Rgb, TerminalPalette};

use crate::domain::color::{dim, resolve};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Normal,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slant {
    Upright,
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decoration {
    None,
    Underline,
    Strikeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    No,
    Yes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Rgb,
    pub bg: Rgb,
    pub weight: Weight,
    pub slant: Slant,
    pub decoration: Decoration,
}

pub fn style(
    fg: Color,
    bg: Color,
    flags: Flags,
    selected: Selected,
    palette: &TerminalPalette,
    overrides: &dyn Fn(usize) -> Option<Rgb>,
) -> Style {
    let mut front = resolve(fg, palette, overrides);
    let mut back = resolve(bg, palette, overrides);
    if flags.contains(Flags::DIM) {
        front = dim(front);
    }
    if flags.contains(Flags::INVERSE) {
        (front, back) = (back, front);
    }
    if flags.contains(Flags::HIDDEN) {
        front = back;
    }
    if selected == Selected::Yes {
        (front, back) = (palette.selection_foreground, palette.selection_background);
    }
    Style {
        fg: front,
        bg: back,
        weight: weight(flags),
        slant: slant(flags),
        decoration: decoration(flags),
    }
}

fn weight(flags: Flags) -> Weight {
    match flags.contains(Flags::BOLD) {
        true => Weight::Bold,
        false => Weight::Normal,
    }
}

fn slant(flags: Flags) -> Slant {
    match flags.contains(Flags::ITALIC) {
        true => Slant::Italic,
        false => Slant::Upright,
    }
}

fn decoration(flags: Flags) -> Decoration {
    match (
        flags.intersects(Flags::ALL_UNDERLINES),
        flags.contains(Flags::STRIKEOUT),
    ) {
        (true, _) => Decoration::Underline,
        (false, true) => Decoration::Strikeout,
        (false, false) => Decoration::None,
    }
}
