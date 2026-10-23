use crate::domain::style::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Single,
    Double,
}

impl Width {
    pub fn cells(self) -> usize {
        match self {
            Width::Single => 1,
            Width::Double => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyph {
    pub line: usize,
    pub column: usize,
    pub character: char,
    pub marks: Vec<char>,
    pub width: Width,
    pub style: Style,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub column: usize,
    pub cells: usize,
    pub text: String,
    pub style: Style,
}

pub fn rows(glyphs: impl IntoIterator<Item = Glyph>, lines: usize) -> Vec<Vec<Run>> {
    let mut rows: Vec<Vec<Run>> = vec![Vec::new(); lines];
    for glyph in glyphs {
        if let Some(row) = rows.get_mut(glyph.line) {
            place(row, glyph);
        }
    }
    rows
}

fn place(row: &mut Vec<Run>, glyph: Glyph) {
    let joins = |run: &Run| {
        run.style == glyph.style
            && run.column + run.cells == glyph.column
            && glyph.character.is_ascii()
            && run.text.is_ascii()
            && glyph.width == Width::Single
            && glyph.marks.is_empty()
    };
    match row.last_mut() {
        Some(run) if joins(run) => {
            run.text.push(glyph.character);
            run.cells += 1;
        }
        _ => row.push(Run {
            column: glyph.column,
            cells: glyph.width.cells(),
            text: std::iter::once(glyph.character)
                .chain(glyph.marks)
                .collect(),
            style: glyph.style,
        }),
    }
}
