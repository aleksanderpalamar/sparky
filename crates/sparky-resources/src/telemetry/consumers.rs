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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub cpu: u64,
    pub io: Option<u64>,
}

impl Usage {
    pub fn cpu(ticks: u64) -> Self {
        Self {
            cpu: ticks,
            io: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Mark {
    ticks: u64,
    bytes: Option<u64>,
    at: Instant,
}

#[derive(Default)]
pub struct Consumers {
    marks: HashMap<Identity, Mark>,
    rates: HashMap<Identity, Usage>,
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
    ) -> Vec<(Consumer, Usage)> {
        let current: Vec<(Consumer, Option<u64>)> = pids
            .iter()
            .filter_map(|pid| Some((read(files, *pid)?, bytes(files, *pid))))
            .collect();
        let alive: HashSet<Identity> = current
            .iter()
            .map(|(consumer, _)| consumer.identity)
            .collect();
        self.marks.retain(|identity, _| alive.contains(identity));
        self.rates.retain(|identity, _| alive.contains(identity));
        for (consumer, bytes) in &current {
            self.advance(consumer, *bytes, now);
        }
        current
            .into_iter()
            .filter_map(|(consumer, _)| {
                let usage = *self.rates.get(&consumer.identity)?;
                Some((consumer, usage))
            })
            .collect()
    }

    fn advance(&mut self, consumer: &Consumer, bytes: Option<u64>, now: Instant) {
        let mark = Mark {
            ticks: consumer.ticks,
            bytes,
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
        let moved = bytes
            .zip(before.bytes)
            .map(|(now, before)| normalized(now.saturating_sub(before), elapsed));
        self.rates.insert(
            consumer.identity,
            Usage {
                cpu: normalized(spent, elapsed),
                io: moved,
            },
        );
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

fn bytes(files: &impl Files, pid: u32) -> Option<u64> {
    let text = files.read(Path::new(&format!("/proc/{pid}/io"))).ok()?;
    let field = |name: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|value| value.trim().parse::<u64>().ok())
    };
    Some(field("read_bytes:")? + field("write_bytes:")?)
}
