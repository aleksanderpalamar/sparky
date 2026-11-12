use std::cmp::Ordering;

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
        compare_folded(self.folded.as_bytes(), other.folded.as_bytes())
            .then_with(|| self.original.cmp(&other.original))
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

pub fn fold(name: &str) -> String {
    if name.is_ascii() {
        return name.to_ascii_lowercase();
    }
    DecomposingNormalizerBorrowed::new_nfd()
        .normalize_iter(name.chars())
        .filter(|character| !COMBINING_MARKS.contains(character))
        .flat_map(char::to_lowercase)
        .collect()
}

fn compare_folded(left: &[u8], right: &[u8]) -> Ordering {
    let (mut at_left, mut at_right) = (0, 0);
    let mut tie = Ordering::Equal;
    while let (Some(&a), Some(&b)) = (left.get(at_left), right.get(at_right)) {
        if a.is_ascii_digit() && b.is_ascii_digit() {
            let (end_left, end_right) = (digits_end(left, at_left), digits_end(right, at_right));
            let (order, zeros) = compare_numbers(
                left.get(at_left..end_left).unwrap_or_default(),
                right.get(at_right..end_right).unwrap_or_default(),
            );
            if order.is_ne() {
                return order;
            }
            tie = tie.then(zeros);
            (at_left, at_right) = (end_left, end_right);
            continue;
        }
        if a != b {
            return a.cmp(&b);
        }
        (at_left, at_right) = (at_left + 1, at_right + 1);
    }
    let rest = |bytes: &[u8], at: usize| bytes.len().saturating_sub(at);
    rest(left, at_left).cmp(&rest(right, at_right)).then(tie)
}

fn digits_end(bytes: &[u8], start: usize) -> usize {
    bytes
        .iter()
        .skip(start)
        .position(|byte| !byte.is_ascii_digit())
        .map_or(bytes.len(), |offset| start + offset)
}

fn compare_numbers(left: &[u8], right: &[u8]) -> (Ordering, Ordering) {
    let (left_value, right_value) = (significant(left), significant(right));
    let order = left_value
        .len()
        .cmp(&right_value.len())
        .then_with(|| left_value.cmp(right_value));
    (order, left.len().cmp(&right.len()))
}

fn significant(run: &[u8]) -> &[u8] {
    let zeros = run.iter().take_while(|digit| **digit == b'0').count();
    run.get(zeros..).unwrap_or_default()
}
