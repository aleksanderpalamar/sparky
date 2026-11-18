use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    CpuBusy,
    MemoryUsed,
    CpuPressure,
    MemoryPressure,
    IoPressure,
    GpuBusy,
    VramUsed,
}

pub const METRICS: [Metric; 7] = [
    Metric::CpuBusy,
    Metric::MemoryUsed,
    Metric::CpuPressure,
    Metric::MemoryPressure,
    Metric::IoPressure,
    Metric::GpuBusy,
    Metric::VramUsed,
];

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Features {
    values: [Option<f32>; METRICS.len()],
}

impl Features {
    pub fn with(mut self, metric: Metric, value: Option<f32>) -> Self {
        self.values[metric as usize] = value
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(0.0, 100.0));
        self
    }

    pub fn get(&self, metric: Metric) -> Option<f32> {
        self.values[metric as usize]
    }
}
