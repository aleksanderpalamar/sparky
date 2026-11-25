use sparky_hardware::pressure::Pressure;
use sparky_hardware::stat::CpuTimes;

use crate::domain::Reading;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySample {
    pub total_kib: u64,
    pub available_kib: u64,
    pub swap_total_kib: u64,
    pub swap_free_kib: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuSample {
    pub busy_percent: Option<u8>,
    pub vram: Option<Vram>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vram {
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pressures {
    pub cpu: Reading<Pressure>,
    pub memory: Reading<Pressure>,
    pub io: Reading<Pressure>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSample {
    pub cpu: Reading<CpuTimes>,
    pub cores: Vec<CpuTimes>,
    pub memory: Reading<MemorySample>,
    pub pressure: Pressures,
    pub gpu: Reading<GpuSample>,
}

pub fn fresh<T>(value: Option<T>) -> Reading<T> {
    value.map_or(Reading::Missing, |value| Reading::Measured {
        value,
        age_ms: 0,
    })
}
