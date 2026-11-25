use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

use sparky_hardware::process::parse_stat;

use crate::domain::Identity;
use crate::ports::Files;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consumer {
    pub identity: Identity,
    pub parent: u32,
    pub name: String,
    pub ticks: u64,
}

const WINDOW: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy)]
struct Mark {
    ticks: u64,
    at: Instant,
}

#[derive(Default)]
pub struct Consumers {
    marks: HashMap<Identity, Mark>,
    rates: HashMap<Identity, u64>,
}

impl Consumers {
    pub fn pause(&mut self) {
        self.marks.clear();
        self.rates.clear();
    }

    pub fn measure(
        &mut self,
        files: &impl Files,
        pids: &[u32],
        now: Instant,
    ) -> Vec<(Consumer, u64)> {
        let current: Vec<Consumer> = pids.iter().filter_map(|pid| read(files, *pid)).collect();
        let alive: HashSet<Identity> = current.iter().map(|consumer| consumer.identity).collect();
        self.marks.retain(|identity, _| alive.contains(identity));
        self.rates.retain(|identity, _| alive.contains(identity));
        for consumer in &current {
            self.advance(consumer, now);
        }
        current
            .into_iter()
            .filter_map(|consumer| {
                let rate = *self.rates.get(&consumer.identity)?;
                Some((consumer, rate))
            })
            .collect()
    }

    fn advance(&mut self, consumer: &Consumer, now: Instant) {
        let mark = Mark {
            ticks: consumer.ticks,
            at: now,
        };
        let Some(before) = self.marks.get(&consumer.identity).copied() else {
            self.marks.insert(consumer.identity, mark);
            return;
        };
        let elapsed = now.saturating_duration_since(before.at);
        if elapsed < WINDOW {
            return;
        }
        let spent = consumer.ticks.saturating_sub(before.ticks);
        self.rates
            .insert(consumer.identity, normalized(spent, elapsed));
        self.marks.insert(consumer.identity, mark);
    }
}

fn normalized(ticks: u64, elapsed: Duration) -> u64 {
    (ticks as f64 * WINDOW.as_secs_f64() / elapsed.as_secs_f64()).round() as u64
}

fn read(files: &impl Files, pid: u32) -> Option<Consumer> {
    let stat = parse_stat(&files.read(Path::new(&format!("/proc/{pid}/stat"))).ok()?)?;
    Some(Consumer {
        identity: Identity {
            pid: stat.pid,
            started: stat.started,
        },
        parent: stat.ppid,
        name: stat.comm,
        ticks: stat.ticks,
    })
}
