use std::collections::BTreeSet;
use std::path::Path;

use sparky_hardware::cpulist::parse_cpu_list;

use crate::ports::Files;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalCpu {
    pub id: u32,
    pub core: u32,
    pub package: u32,
    pub siblings: Vec<u32>,
    pub last_cache: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Topology {
    pub cpus: Vec<LogicalCpu>,
}

impl Topology {
    pub fn logical(&self) -> usize {
        self.cpus.len()
    }

    pub fn physical(&self) -> usize {
        let cores: BTreeSet<(u32, u32)> = self
            .cpus
            .iter()
            .map(|cpu| (cpu.package, cpu.core))
            .collect();
        cores.len()
    }

    pub fn has_smt(&self) -> bool {
        self.cpus.iter().any(|cpu| cpu.siblings.len() > 1)
    }

    pub fn last_core(&self) -> Vec<u32> {
        let Some(last) = self.cpus.iter().max_by_key(|cpu| (cpu.package, cpu.core)) else {
            return Vec::new();
        };
        let mut cpus: Vec<u32> = self
            .cpus
            .iter()
            .filter(|cpu| (cpu.package, cpu.core) == (last.package, last.core))
            .map(|cpu| cpu.id)
            .collect();
        cpus.sort_unstable();
        cpus
    }

    pub fn siblings(&self, id: u32) -> &[u32] {
        self.cpus
            .iter()
            .find(|cpu| cpu.id == id)
            .map_or(&[], |cpu| cpu.siblings.as_slice())
    }
}

pub fn read_topology(files: &impl Files) -> Option<Topology> {
    let online = files
        .read(Path::new("/sys/devices/system/cpu/online"))
        .ok()?;
    let cpus = parse_cpu_list(&online)?
        .into_iter()
        .map(|id| logical(files, id))
        .collect::<Option<Vec<_>>>()?;
    Some(Topology { cpus })
}

fn logical(files: &impl Files, id: u32) -> Option<LogicalCpu> {
    let base = format!("/sys/devices/system/cpu/cpu{id}");
    let text = |name: &str| files.read(Path::new(&format!("{base}/{name}"))).ok();
    let number = |name: &str| text(name)?.trim().parse::<u32>().ok();
    Some(LogicalCpu {
        id,
        core: number("topology/core_id")?,
        package: number("topology/physical_package_id")?,
        siblings: parse_cpu_list(&text("topology/thread_siblings_list")?)?,
        last_cache: last_cache(files, &base).unwrap_or_default(),
    })
}

fn last_cache(files: &impl Files, base: &str) -> Option<Vec<u32>> {
    let directory = format!("{base}/cache");
    let mut indexes = files.entries(Path::new(&directory)).ok()?;
    indexes.retain(|name| name.starts_with("index"));
    indexes.sort();
    let last = indexes.last()?;
    let shared = files
        .read(Path::new(&format!("{directory}/{last}/shared_cpu_list")))
        .ok()?;
    parse_cpu_list(&shared)
}
