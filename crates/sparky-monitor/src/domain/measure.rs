use std::collections::HashMap;
use std::time::Duration;

use super::cpu::usage;
use super::memory::Memory;
use super::snapshot::{CpuInfo, DiskSample, GpuDevice, ProcessSample, Snapshot};
use super::usage::{ProcessRow, Usage};

#[derive(Debug, Clone, PartialEq)]
pub struct DiskReading {
    pub sample: DiskSample,
    pub read: f64,
    pub write: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetReading {
    pub name: String,
    pub received: f64,
    pub sent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub cpu: f32,
    pub cores: Vec<f32>,
    pub cpu_info: CpuInfo,
    pub memory: Option<Memory>,
    pub disks: Vec<DiskReading>,
    pub networks: Vec<NetReading>,
    pub gpus: Vec<GpuDevice>,
    pub processes: Vec<ProcessRow>,
}

pub fn measure(previous: &Snapshot, current: &Snapshot) -> Reading {
    let elapsed = current.at.saturating_duration_since(previous.at);
    let cores = current
        .cores
        .iter()
        .enumerate()
        .map(|(index, core)| {
            previous
                .cores
                .get(index)
                .map_or(0.0, |before| usage(*before, *core))
        })
        .collect();
    Reading {
        cpu: usage(previous.cpu, current.cpu),
        cores,
        cpu_info: current.cpu_info.clone(),
        memory: current.memory,
        disks: disks(previous, current, elapsed),
        networks: networks(previous, current, elapsed),
        gpus: current.gpus.clone(),
        processes: processes(previous, current, elapsed),
    }
}

pub fn per_second(before: u64, after: u64, elapsed: Duration) -> f64 {
    match elapsed.as_secs_f64() {
        seconds if seconds > 0.0 => after.saturating_sub(before) as f64 / seconds,
        _ => 0.0,
    }
}

fn disks(previous: &Snapshot, current: &Snapshot, elapsed: Duration) -> Vec<DiskReading> {
    current
        .disks
        .iter()
        .map(|disk| {
            let before = previous
                .disks
                .iter()
                .find(|old| old.counters.name == disk.counters.name)
                .map_or(&disk.counters, |old| &old.counters);
            DiskReading {
                read: per_second(before.read, disk.counters.read, elapsed),
                write: per_second(before.written, disk.counters.written, elapsed),
                sample: disk.clone(),
            }
        })
        .collect()
}

fn networks(previous: &Snapshot, current: &Snapshot, elapsed: Duration) -> Vec<NetReading> {
    current
        .networks
        .iter()
        .map(|net| {
            let before = previous
                .networks
                .iter()
                .find(|old| old.name == net.name)
                .unwrap_or(net);
            NetReading {
                name: net.name.clone(),
                received: per_second(before.received, net.received, elapsed),
                sent: per_second(before.sent, net.sent, elapsed),
            }
        })
        .collect()
}

fn processes(previous: &Snapshot, current: &Snapshot, elapsed: Duration) -> Vec<ProcessRow> {
    let before: HashMap<u32, &ProcessSample> = previous
        .processes
        .iter()
        .map(|process| (process.stat.pid, process))
        .collect();
    let machine = current.cpu.total.saturating_sub(previous.cpu.total);
    current
        .processes
        .iter()
        .map(|process| {
            let old = before.get(&process.stat.pid).copied().unwrap_or(process);
            row(old, process, machine, elapsed)
        })
        .collect()
}

fn row(old: &ProcessSample, new: &ProcessSample, machine: u64, elapsed: Duration) -> ProcessRow {
    let ticks = new.stat.ticks.saturating_sub(old.stat.ticks);
    let cpu = match machine {
        0 => 0.0,
        machine => (ticks as f64 / machine as f64 * 100.0) as f32,
    };
    let io = |sample: &ProcessSample| sample.io.map_or(0, |(read, write)| read + write);
    let gpu = new.gpu.map(|gpu| {
        let engine = gpu
            .engine_ns
            .saturating_sub(old.gpu.map_or(gpu.engine_ns, |old| old.engine_ns));
        let wall = elapsed.as_nanos().max(1) as f64;
        ((engine as f64 / wall * 100.0) as f32).min(100.0)
    });
    ProcessRow {
        pid: new.stat.pid,
        ppid: new.stat.ppid,
        name: new.name.clone(),
        program: new.program.clone(),
        user: new.user,
        usage: Usage {
            cpu,
            memory: new.memory,
            disk: per_second(io(old), io(new), elapsed),
            gpu,
            vram: new.gpu.map(|gpu| gpu.vram),
        },
    }
}
