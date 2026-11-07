use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Status, Value};
use crate::render::visual::ansi;
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::Paint;
use crate::render::visual::swatch;
use crate::render::{label, shown};

const SPACING: usize = 1;

pub(crate) fn block(report: &Report, paint: &Paint, room: usize) -> Vec<Line> {
    let mut lines = rows(report);
    let Some(samples) = swatch::samples(report) else {
        return lines;
    };
    let row = swatch::row(samples, paint, room);
    if row.width() > 0 {
        lines.extend([Line::blank(), row]);
    }
    lines
}

pub(crate) fn rows(report: &Report) -> Vec<Line> {
    let entries: Vec<&Entry> = report
        .entries
        .iter()
        .filter(|entry| !swatch::is_palette(entry))
        .collect();
    let column = entries
        .iter()
        .map(|entry| ansi::width(&caption(entry.field)))
        .max()
        .unwrap_or(0);
    entries
        .into_iter()
        .flat_map(|entry| entry_rows(entry, column))
        .collect()
}

fn entry_rows(entry: &Entry, column: usize) -> Vec<Line> {
    let caption = caption(entry.field);
    let first = Line::new(vec![
        Segment::new(caption.clone(), Tone::Label),
        Segment::space(column - ansi::width(&caption) + SPACING),
    ]);
    values(&entry.status)
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let lead = match index {
                0 => first.clone(),
                _ => Line::new(vec![Segment::space(column + SPACING)]),
            };
            lead.then(Line::new(vec![Segment::new(value, Tone::Value)]))
        })
        .collect()
}

fn caption(field: Field) -> String {
    format!("{}:", clean(&label(field)))
}

fn values(status: &Status) -> Vec<String> {
    match status.value() {
        Some(Value::Lines(lines)) if !lines.is_empty() => {
            lines.iter().map(|line| clean(line)).collect()
        }
        _ => vec![clean(&shown(status))],
    }
}

fn clean(text: &str) -> String {
    ansi::strip(text).trim().to_owned()
}
