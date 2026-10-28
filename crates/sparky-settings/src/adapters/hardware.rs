use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::hardware::{self, Hardware};
use crate::domain::pci;

const DATABASES: [&str; 3] = [
    "/usr/share/hwdata/pci.ids",
    "/usr/share/misc/pci.ids",
    "/usr/share/pci.ids",
];

pub fn load() -> Hardware {
    let database = DATABASES
        .into_iter()
        .find_map(|path| fs::read_to_string(path).ok())
        .unwrap_or_default();
    from_paths(
        Path::new("/proc"),
        Path::new("/sys/bus/pci/devices"),
        &database,
    )
}

fn from_paths(proc: &Path, devices: &Path, database: &str) -> Hardware {
    let mut paths: Vec<PathBuf> = fs::read_dir(devices)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    Hardware {
        processor: read(&proc.join("cpuinfo"))
            .as_deref()
            .and_then(hardware::processor),
        gpus: paths
            .iter()
            .filter_map(|path| gpu(path, database))
            .collect(),
        memory_kib: read(&proc.join("meminfo"))
            .as_deref()
            .and_then(hardware::memory_kib),
    }
}

fn gpu(path: &Path, database: &str) -> Option<String> {
    if hex(&path.join("class"))? >> 16 != 0x03 {
        return None;
    }
    if let Some(name) = read(&path.join("product_name")) {
        return Some(name);
    }
    let vendor = hex(&path.join("vendor"))?.try_into().ok()?;
    let device = hex(&path.join("device"))?.try_into().ok()?;
    Some(pci::name(database, vendor, device))
}

fn read(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn hex(path: &Path) -> Option<u32> {
    let text = read(path)?;
    u32::from_str_radix(text.strip_prefix("0x").unwrap_or(&text), 16).ok()
}

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;
