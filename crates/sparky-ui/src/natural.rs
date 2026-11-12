use std::cmp::Ordering;
use std::iter::Peekable;
use std::str::Chars;

use icu_normalizer::DecomposingNormalizerBorrowed;

const COMBINING_MARKS: std::ops::RangeInclusive<char> = '\u{0300}'..='\u{036F}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NaturalKey {
    folded: String,
    original: String,
}

impl NaturalKey {
    pub fn new(name: &str) -> Self {
        Self {
            folded: fold(name),
            original: name.to_owned(),
        }
    }
}

impl Ord for NaturalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_folded(&self.folded, &other.folded).then_with(|| self.original.cmp(&other.original))
    }
}

impl PartialOrd for NaturalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn compare(left: &str, right: &str) -> Ordering {
    NaturalKey::new(left).cmp(&NaturalKey::new(right))
}

fn fold(name: &str) -> String {
    DecomposingNormalizerBorrowed::new_nfd()
        .normalize_iter(name.chars())
        .filter(|character| !COMBINING_MARKS.contains(character))
        .flat_map(char::to_lowercase)
        .collect()
}

fn compare_folded(left: &str, right: &str) -> Ordering {
    let (mut left, mut right) = (left.chars().peekable(), right.chars().peekable());
    let mut tie = Ordering::Equal;
    loop {
        let step = match (left.peek().copied(), right.peek().copied()) {
            (None, None) => return tie,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a), Some(b)) if a.is_ascii_digit() && b.is_ascii_digit() => {
                let (order, zeros) = compare_numbers(&digits(&mut left), &digits(&mut right));
                tie = tie.then(zeros);
                order
            }
            (Some(a), Some(b)) => {
                left.next();
                right.next();
                a.cmp(&b)
            }
        };
        if step.is_ne() {
            return step;
        }
    }
}

fn digits(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut run = String::new();
    while let Some(digit) = chars.next_if(char::is_ascii_digit) {
        run.push(digit);
    }
    run
}

fn compare_numbers(left: &str, right: &str) -> (Ordering, Ordering) {
    let (left_value, right_value) = (left.trim_start_matches('0'), right.trim_start_matches('0'));
    let order = left_value
        .len()
        .cmp(&right_value.len())
        .then_with(|| left_value.cmp(right_value));
    (order, left.len().cmp(&right.len()))
}
