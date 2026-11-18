use std::path::Path;

use crate::application::Budget;
use crate::ports::Files;

pub const NORMAL_RSS_KIB: u64 = 20 * 1024;
pub const SENSITIVE_CPU_PERCENT: f64 = 0.1;
pub const NORMAL_CPU_PERCENT: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Usage {
    pub cpu_seconds: f64,
    pub rss_kib: u64,
}

pub fn usage(files: &impl Files, ticks_per_second: f64) -> Option<Usage> {
    let stat = files.read(Path::new("/proc/self/stat")).ok()?;
    let status = files.read(Path::new("/proc/self/status")).ok()?;
    let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    let ticks = fields.get(11)?.parse::<f64>().ok()? + fields.get(12)?.parse::<f64>().ok()?;
    let rss_kib = status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.split_whitespace().next()?.parse().ok())?;
    Some(Usage {
        cpu_seconds: ticks / ticks_per_second,
        rss_kib,
    })
}

pub fn assess(before: Usage, after: Usage, elapsed_seconds: f64, sensitive: bool) -> Budget {
    let cpu = (after.cpu_seconds - before.cpu_seconds).max(0.0) / elapsed_seconds.max(1e-3) * 100.0;
    let ceiling = if sensitive {
        SENSITIVE_CPU_PERCENT
    } else {
        NORMAL_CPU_PERCENT
    };
    match cpu > ceiling || after.rss_kib > NORMAL_RSS_KIB {
        true => Budget::Exceeded,
        false => Budget::Within,
    }
}
