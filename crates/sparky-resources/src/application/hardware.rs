use std::path::Path;

use sparky_hardware::procfs::{processor, threads, total_memory_kib};

use crate::persistence::checkpoint::checksum;
use crate::ports::Files;

const GIB_IN_KIB: u64 = 1024 * 1024;

pub fn fingerprint(files: &impl Files, gpu: bool) -> u64 {
    let cpuinfo = files.read(Path::new("/proc/cpuinfo")).unwrap_or_default();
    let meminfo = files.read(Path::new("/proc/meminfo")).unwrap_or_default();
    let memory = total_memory_kib(&meminfo).map_or(0, |kib| kib.div_ceil(GIB_IN_KIB));
    let summary = format!(
        "{}|{}|{memory}|{gpu}",
        processor(&cpuinfo).unwrap_or_default(),
        threads(&cpuinfo)
    );
    checksum(summary.as_bytes())
}
