use unicode_width::UnicodeWidthChar;

use crate::render::visual::segment::{Line, Segment};

const ELLIPSIS: char = '…';

pub(crate) fn clip(line: Line, width: usize) -> Line {
    if line.width() <= width {
        return line;
    }
    let Some(mut room) = width.checked_sub(1) else {
        return Line::default();
    };
    let mut kept = Vec::new();
    for segment in line.segments {
        let segment_width = segment.width();
        if segment_width <= room {
            room -= segment_width;
            kept.push(segment);
            continue;
        }
        kept.push(cut(&segment, room));
        break;
    }
    Line::new(kept)
}

fn cut(segment: &Segment, room: usize) -> Segment {
    let mut text = prefix(&segment.text, room);
    text.push(ELLIPSIS);
    Segment::new(text, segment.tone)
}

fn prefix(text: &str, room: usize) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|character| {
            used += character.width().unwrap_or(0);
            used <= room
        })
        .collect()
}
