use std::ops::Range;

use super::Listing;
use crate::domain::entry::Meta;

impl Listing {
    pub fn pending(&self, range: Range<usize>) -> Vec<(usize, String)> {
        range
            .filter_map(|index| self.shown.get(index).copied())
            .filter_map(|at| {
                self.entries
                    .get(at)
                    .filter(|entry| entry.meta == Meta::Pending)
                    .map(|entry| (at, entry.name.clone()))
            })
            .collect()
    }

    pub fn learn(&mut self, at: usize, meta: Meta) {
        if let Some(entry) = self.entries.get_mut(at) {
            entry.meta = meta;
        }
    }
}
