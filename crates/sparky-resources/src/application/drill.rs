use super::encoding::{Example, WINDOW};
use crate::neural::{Adam, Hybrid};

pub const LEAK_GUARD: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Division {
    pub split: usize,
    pub last: usize,
}

pub fn divide(rows: usize) -> Division {
    let split = rows * 7 / 10;
    Division {
        split,
        last: split.saturating_sub(LEAK_GUARD).max(WINDOW),
    }
}

pub fn practise<'a>(
    model: &mut Hybrid,
    adam: &mut Adam,
    lessons: impl IntoIterator<Item = &'a Example>,
    decay: f32,
) -> usize {
    let mut learned = 0;
    for example in lessons {
        model.accumulate(&example.window, &example.context, &example.targets);
        learned += 1;
    }
    if learned > 0 {
        adam.step(&mut model.params);
        model.settle(decay);
    }
    learned
}
