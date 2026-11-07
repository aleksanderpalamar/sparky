use std::fs;
use std::path::{Path, PathBuf};

use sparky_hardware::{gpu, pci, procfs};

use crate::domain::hardware::Hardware;

pub fn load() -> Hardware {
    let database = pci::database(|path| fs::read_to_string(path).ok());
    from_paths(
        Path::new("/proc"),
        Path::new("/sys/bus/pci/devices"),
        &database,
    )
}

fn from_paths(proc: &Path, devices: &Path, database: &str) -> Hardware {
    let paths: Vec<PathBuf> = fs::read_dir(devices)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    Hardware {
        processor: read(&proc.join("cpuinfo"))
            .as_deref()
            .and_then(procfs::processor),
        gpus: gpu::names(&paths, database, read),
        memory_kib: read(&proc.join("meminfo"))
            .as_deref()
            .and_then(procfs::total_memory_kib),
    }
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;
