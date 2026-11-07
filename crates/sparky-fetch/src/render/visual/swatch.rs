use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Status, Value, hex};
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::Paint;

const BLOCK: &str = "███";
const SEPARATOR: usize = 1;

pub(crate) fn palette(report: &Report) -> Option<&[Rgb]> {
    colors(report.status(Field::Colors)?)
}

pub(crate) fn is_palette(entry: &Entry) -> bool {
    entry.field == Field::Colors && colors(&entry.status).is_some()
}

pub(crate) fn samples(report: &Report) -> Option<&[Rgb]> {
    palette(report)?
        .get(1..)
        .filter(|samples| !samples.is_empty())
}

pub(crate) fn row(samples: &[Rgb], paint: &Paint, room: usize) -> Line {
    let mut segments = Vec::new();
    let mut used = 0;
    for color in samples {
        let swatch = Segment::new(face(*color, paint), Tone::Swatch(*color));
        let gap = if segments.is_empty() { 0 } else { SEPARATOR };
        if used + gap + swatch.width() > room {
            break;
        }
        used += gap + swatch.width();
        if gap > 0 {
            segments.push(Segment::space(gap));
        }
        segments.push(swatch);
    }
    Line::new(segments)
}

fn colors(status: &Status) -> Option<&[Rgb]> {
    match status.value()? {
        Value::Colors(colors) => Some(colors),
        Value::Text(_) | Value::Lines(_) => None,
    }
}

fn face(color: Rgb, paint: &Paint) -> String {
    match paint {
        Paint::Plain => hex(color),
        Paint::TrueColor(_) => BLOCK.to_owned(),
    }
}
