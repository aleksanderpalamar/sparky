use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, Instant};

use sparky_hardware::process::parse_stat;

use crate::context::Process;
use crate::ports::Files;

const KERNEL_THREADS: u32 = 2;
const REFRESH_ALL: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct ProcessTable {
    known: HashMap<u32, Option<Process>>,
    refreshed: Option<Instant>,
}

impl ProcessTable {
    pub fn update(&mut self, files: &impl Files, now: Instant) -> Option<Vec<Process>> {
        let pids = pids(files)?;
        if self
            .refreshed
            .is_none_or(|refreshed| now.duration_since(refreshed) >= REFRESH_ALL)
        {
            self.known.clear();
            self.refreshed = Some(now);
        }
        self.known.retain(|pid, _| pids.contains(pid));
        for pid in pids {
            self.known.entry(pid).or_insert_with(|| read(files, pid));
        }
        let mut processes: Vec<Process> = self.known.values().flatten().cloned().collect();
        processes.sort_by_key(|process| process.pid);
        Some(processes)
    }
}

pub fn scan(files: &impl Files) -> Option<Vec<Process>> {
    ProcessTable::default().update(files, Instant::now())
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

fn read(files: &impl Files, pid: u32) -> Option<Process> {
    let text = files.read(Path::new(&format!("/proc/{pid}/stat"))).ok()?;
    let stat = parse_stat(&text)
        .filter(|stat| stat.pid != KERNEL_THREADS && stat.ppid != KERNEL_THREADS)?;
    Some(Process {
        pid: stat.pid,
        parent: stat.ppid,
        name: stat.comm,
    })
}
