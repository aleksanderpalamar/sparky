use std::ops::Range;

use crate::application::encoding::{Sample, WINDOW};
use crate::application::reference::continuous;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holdout {
    pub training: Range<usize>,
    pub reserved: Range<usize>,
}

pub fn sessions(rows: &[Sample]) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut start = 0;
    for index in 1..rows.len() {
        if !continuous(&rows[index - 1], &rows[index]) {
            found.push(start..index);
            start = index;
        }
    }
    if !rows.is_empty() {
        found.push(start..rows.len());
    }
    found
}

pub fn reserve(rows: &[Sample], share: f64) -> Option<Holdout> {
    let all = sessions(rows);
    let wanted = (rows.len() as f64 * share.clamp(0.0, 1.0)).ceil() as usize;
    let start = match all.len() {
        0 => return None,
        1 => rows.len() - wanted.min(rows.len()),
        _ => whole(&all, rows.len(), wanted),
    };
    let holdout = Holdout {
        training: 0..start,
        reserved: start..rows.len(),
    };
    (holdout.training.len() > WINDOW && holdout.reserved.len() > WINDOW).then_some(holdout)
}

pub fn overlap(earlier: &Range<usize>, later: &Range<usize>) -> f64 {
    let shared = later
        .end
        .min(earlier.end)
        .saturating_sub(later.start.max(earlier.start));
    match later.len() {
        0 => 0.0,
        length => shared as f64 / length as f64,
    }
}

fn whole(all: &[Range<usize>], total: usize, wanted: usize) -> usize {
    let mut start = total;
    for session in all.iter().skip(1).rev() {
        start = session.start;
        if total - start >= wanted {
            break;
        }
    }
    start
}
