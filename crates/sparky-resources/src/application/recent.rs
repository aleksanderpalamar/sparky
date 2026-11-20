use super::encoding::{Sample, WINDOW, window};
use super::reference::{Base, Misses, misses};

const KEEP: f64 = 90.0;

#[derive(Debug, Default)]
pub struct Recent {
    samples: Vec<Sample>,
    bases: Vec<Base>,
}

impl Recent {
    pub fn push(&mut self, sample: Sample, base: Base) {
        self.samples.push(sample);
        self.bases.push(base);
        let stale = self
            .samples
            .partition_point(|old| old.at < sample.at - KEEP)
            .min(self.samples.len().saturating_sub(WINDOW));
        self.samples.drain(..stale);
        self.bases.drain(..stale);
    }

    pub fn window(&self) -> Option<Vec<Vec<f32>>> {
        let start = self.samples.len().checked_sub(WINDOW)?;
        Some(window(&self.samples[start..]))
    }

    pub fn misses(&self) -> Misses {
        misses(&self.samples, &self.bases)
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}
