use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ports::Files;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Footprint {
    pub cpu_percent: f64,
    pub rss_kib: u64,
    pub limit_kib: Option<u64>,
}

pub fn memory_limit(files: &impl Files) -> Option<u64> {
    let membership = files.read(Path::new("/proc/self/cgroup")).ok()?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?
        .trim();
    let limit = files
        .read(Path::new(&format!("/sys/fs/cgroup{group}/memory.max")))
        .ok()?;
    limit.trim().parse::<u64>().ok().map(|bytes| bytes / 1024)
}
