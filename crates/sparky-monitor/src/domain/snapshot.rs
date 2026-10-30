use std::time::Instant;

use super::cpu::CpuTimes;
use super::disk::DiskCounters;
use super::gpu::GpuUse;
use super::memory::Memory;
use super::network::NetCounters;
use super::process::ProcStat;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CpuInfo {
    pub model: Option<String>,
    pub frequency: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskSample {
    pub counters: DiskCounters,
    pub model: Option<String>,
    pub capacity: u64,
    pub used: u64,
    pub mounts: Vec<String>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuDevice {
    pub name: String,
    pub busy: Option<f32>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessSample {
    pub stat: ProcStat,
    pub name: String,
    pub program: String,
    pub user: Option<u32>,
    pub memory: u64,
    pub io: Option<(u64, u64)>,
    pub gpu: Option<GpuUse>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub at: Instant,
    pub cpu: CpuTimes,
    pub cores: Vec<CpuTimes>,
    pub cpu_info: CpuInfo,
    pub memory: Option<Memory>,
    pub disks: Vec<DiskSample>,
    pub networks: Vec<NetCounters>,
    pub gpus: Vec<GpuDevice>,
    pub processes: Vec<ProcessSample>,
}
