use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{RenderableContent, Term, TermMode};
use alacritty_terminal::vte::ansi::CursorShape;
use sparky_theme::{Rgb, TerminalPalette};

use crate::adapters::listener::Listener;
use crate::domain::runs::{Glyph, Width, rows};
use crate::domain::screen::{CursorSpot, Screen};
use crate::domain::style::{Selected, style};

const SPACERS: Flags = Flags::WIDE_CHAR_SPACER.union(Flags::LEADING_WIDE_CHAR_SPACER);

pub fn snapshot(term: &Term<Listener>, palette: &TerminalPalette) -> Screen {
    let lines = term.screen_lines();
    let RenderableContent {
        display_iter,
        selection,
        cursor,
        display_offset,
        colors,
        mode,
    } = term.renderable_content();
    let offset = display_offset as i32;
    let overrides = |index: usize| colors[index].map(|color| Rgb::new(color.r, color.g, color.b));
    let glyphs = display_iter
        .filter(|indexed| !indexed.cell.flags.intersects(SPACERS))
        .filter_map(|indexed| {
            let line = usize::try_from(indexed.point.line.0 + offset).ok()?;
            let selected = match selection.is_some_and(|range| range.contains(indexed.point)) {
                true => Selected::Yes,
                false => Selected::No,
            };
            let width = match indexed.cell.flags.contains(Flags::WIDE_CHAR) {
                true => Width::Double,
                false => Width::Single,
            };
            let style = style(
                indexed.cell.fg,
                indexed.cell.bg,
                indexed.cell.flags,
                selected,
                palette,
                &overrides,
            );
            Some(Glyph {
                line,
                column: indexed.point.column.0,
                character: indexed.cell.c,
                width,
                style,
            })
        });
    let rows = rows(glyphs, lines);
    let visible = mode.contains(TermMode::SHOW_CURSOR) && cursor.shape != CursorShape::Hidden;
    let cursor = visible.then(|| {
        let line = usize::try_from(cursor.point.line.0 + offset)
            .ok()
            .filter(|line| *line < lines)?;
        Some(CursorSpot {
            line,
            column: cursor.point.column.0,
            character: term.grid()[cursor.point].c,
        })
    });
    Screen {
        rows,
        cursor: cursor.flatten(),
    }
}
