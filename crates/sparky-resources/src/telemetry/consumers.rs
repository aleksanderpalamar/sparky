use std::collections::HashMap;
use std::path::Path;

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

#[derive(Default)]
pub struct Consumers {
    previous: HashMap<Identity, u64>,
}

impl Consumers {
    pub fn measure(&mut self, files: &impl Files, pids: &[u32]) -> Vec<(Consumer, u64)> {
        let current: Vec<Consumer> = pids.iter().filter_map(|pid| read(files, *pid)).collect();
        let deltas = current
            .iter()
            .map(|consumer| {
                let before = self
                    .previous
                    .get(&consumer.identity)
                    .copied()
                    .unwrap_or(consumer.ticks);
                (consumer.clone(), consumer.ticks.saturating_sub(before))
            })
            .collect();
        self.previous = current
            .into_iter()
            .map(|consumer| (consumer.identity, consumer.ticks))
            .collect();
        deltas
    }
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
