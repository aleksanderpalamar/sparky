use std::cmp::Ordering;

use sparky_ui::natural::NaturalKey;

use crate::domain::entry::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    #[default]
    Name,
    Size,
    Modified,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Order {
    pub key: SortKey,
    pub direction: SortDirection,
}

impl Order {
    pub fn new(key: SortKey, direction: SortDirection) -> Self {
        Self { key, direction }
    }

    pub fn next_key(self) -> Self {
        let key = match self.key {
            SortKey::Name => SortKey::Size,
            SortKey::Size => SortKey::Modified,
            SortKey::Modified => SortKey::Type,
            SortKey::Type => SortKey::Name,
        };
        Self { key, ..self }
    }

    pub fn reversed(self) -> Self {
        let direction = match self.direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
        Self { direction, ..self }
    }

    pub fn choose(self, key: SortKey) -> Self {
        match key == self.key {
            true => self.reversed(),
            false => Self::new(key, SortDirection::Ascending),
        }
    }

    pub fn needs_details(self) -> bool {
        matches!(self.key, SortKey::Size | SortKey::Modified)
    }
}

pub fn sort(entries: &mut Vec<Entry>, order: Order) {
    let mut keyed: Vec<(NaturalKey, Entry)> = std::mem::take(entries)
        .into_iter()
        .map(|entry| (NaturalKey::new(&entry.name), entry))
        .collect();
    keyed.sort_by(|(left_name, left), (right_name, right)| {
        right
            .is_directory()
            .cmp(&left.is_directory())
            .then_with(|| by_key(left, right, order))
            .then_with(|| by_name(left_name, right_name, order))
    });
    *entries = keyed.into_iter().map(|(_, entry)| entry).collect();
}

fn by_name(left: &NaturalKey, right: &NaturalKey, order: Order) -> Ordering {
    match (order.key, order.direction) {
        (SortKey::Name, SortDirection::Descending) => right.cmp(left),
        _ => left.cmp(right),
    }
}

fn by_key(left: &Entry, right: &Entry, order: Order) -> Ordering {
    if left.is_directory() && order.key == SortKey::Size {
        return Ordering::Equal;
    }
    match order.key {
        SortKey::Name => Ordering::Equal,
        SortKey::Size => known(left.size(), right.size(), order.direction),
        SortKey::Modified => known(left.modified(), right.modified(), order.direction),
        SortKey::Type => match order.direction {
            SortDirection::Ascending => extension(left).cmp(&extension(right)),
            SortDirection::Descending => extension(right).cmp(&extension(left)),
        },
    }
}

fn known<T: Ord>(left: Option<T>, right: Option<T>, direction: SortDirection) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => match direction {
            SortDirection::Ascending => left.cmp(&right),
            SortDirection::Descending => right.cmp(&left),
        },
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn extension(entry: &Entry) -> String {
    match entry.name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => extension.to_lowercase(),
        _ => String::new(),
    }
}
