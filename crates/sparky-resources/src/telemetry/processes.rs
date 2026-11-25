use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

use sparky_hardware::process::parse_stat;

use crate::context::Process;
use crate::ports::Files;

const KERNEL_THREADS: u32 = 2;
const ROTATION: usize = 8;
const DECISION_EVERY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sweep {
    Routine,
    Decision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Entry {
    Listed(Process),
    Hidden,
}

#[derive(Default)]
pub struct ProcessTable {
    known: HashMap<u32, Entry>,
    turn: usize,
    swept: Option<Instant>,
}

impl ProcessTable {
    pub fn update(
        &mut self,
        files: &impl Files,
        now: Instant,
        sweep: Sweep,
    ) -> Option<Vec<Process>> {
        let pids = pids(files)?;
        self.known.retain(|pid, _| pids.contains(pid));
        let stale = self.stale(now, sweep);
        let fresh = pids.iter().filter(|pid| !self.known.contains_key(pid));
        let due: Vec<u32> = fresh.copied().chain(stale).collect();
        for pid in due {
            match read(files, pid) {
                Some(entry) => self.known.insert(pid, entry),
                None => self.known.remove(&pid),
            };
        }
        let mut processes: Vec<Process> = self
            .known
            .values()
            .filter_map(|entry| match entry {
                Entry::Listed(process) => Some(process.clone()),
                Entry::Hidden => None,
            })
            .collect();
        processes.sort_by_key(|process| process.pid);
        Some(processes)
    }

    fn stale(&mut self, now: Instant, sweep: Sweep) -> Vec<u32> {
        let mut known: Vec<u32> = self.known.keys().copied().collect();
        known.sort_unstable();
        let due = self
            .swept
            .is_none_or(|swept| now.duration_since(swept) >= DECISION_EVERY);
        if sweep == Sweep::Decision && due {
            self.swept = Some(now);
            return known;
        }
        let slice = known.len().div_ceil(ROTATION);
        let start = (self.turn % ROTATION) * slice;
        self.turn = self.turn.wrapping_add(1);
        known.into_iter().skip(start).take(slice).collect()
    }
}

pub fn scan(files: &impl Files) -> Option<Vec<Process>> {
    ProcessTable::default().update(files, Instant::now(), Sweep::Routine)
}

fn pids(files: &impl Files) -> Option<HashSet<u32>> {
    let entries = files.entries(Path::new("/proc")).ok()?;
    Some(
        entries
            .iter()
            .filter_map(|name| name.parse().ok())
            .collect(),
    )
}

fn read(files: &impl Files, pid: u32) -> Option<Entry> {
    let text = files.read(Path::new(&format!("/proc/{pid}/stat"))).ok()?;
    let stat = parse_stat(&text)?;
    if stat.pid == KERNEL_THREADS || stat.ppid == KERNEL_THREADS {
        return Some(Entry::Hidden);
    }
    Some(Entry::Listed(Process {
        pid: stat.pid,
        parent: stat.ppid,
        name: stat.comm,
    }))
}
