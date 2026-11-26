use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::telemetry::consumers::{Consumer, Usage};

const FIRST: Duration = Duration::from_secs(600);
const LONGEST: Duration = Duration::from_secs(3600);
const MEMORY: usize = 64;

#[derive(Debug, Clone, Copy)]
struct Strikes {
    count: u32,
    until: Instant,
}

#[derive(Debug, Default)]
pub struct Backoff {
    strikes: HashMap<String, Strikes>,
}

impl Backoff {
    pub fn failed(&mut self, name: &str, now: Instant) {
        self.strikes
            .retain(|_, strikes| now < strikes.until + LONGEST);
        if !self.strikes.contains_key(name) && self.strikes.len() >= MEMORY {
            self.forget_oldest();
        }
        let count = self
            .strikes
            .get(name)
            .map_or(1, |strikes| strikes.count.saturating_add(1));
        let until = now + pause(count);
        self.strikes
            .insert(name.to_owned(), Strikes { count, until });
    }

    pub fn worked(&mut self, name: &str) {
        self.strikes.remove(name);
    }

    pub fn resting(&self, name: &str, now: Instant) -> bool {
        self.strikes
            .get(name)
            .is_some_and(|strikes| now < strikes.until)
    }

    pub fn eligible(
        &self,
        consumers: &[(Consumer, Usage)],
        now: Instant,
    ) -> Vec<(Consumer, Usage)> {
        consumers
            .iter()
            .filter(|(consumer, _)| !self.resting(&consumer.name, now))
            .cloned()
            .collect()
    }

    pub fn remembered(&self) -> usize {
        self.strikes.len()
    }

    fn forget_oldest(&mut self) {
        let oldest = self
            .strikes
            .iter()
            .min_by_key(|(_, strikes)| strikes.until)
            .map(|(name, _)| name.clone());
        if let Some(name) = oldest {
            self.strikes.remove(&name);
        }
    }
}

fn pause(count: u32) -> Duration {
    let doublings = count.saturating_sub(1).min(6);
    FIRST.saturating_mul(1 << doublings).min(LONGEST)
}
