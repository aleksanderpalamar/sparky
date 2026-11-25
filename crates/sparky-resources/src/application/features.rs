use sparky_hardware::stat::{CpuTimes, usage};

use crate::domain::{Features, Metric};
use crate::telemetry::SystemSample;

pub fn features(previous_cpu: Option<CpuTimes>, sample: &SystemSample) -> Features {
    let cpu = previous_cpu
        .zip(sample.cpu.measured().copied())
        .map(|(before, after)| usage(before, after));
    let memory = sample
        .memory
        .measured()
        .filter(|memory| memory.total_kib > 0);
    let gpu = sample.gpu.measured();
    let stall = |reading: &crate::domain::Reading<sparky_hardware::pressure::Pressure>| {
        reading.measured().map(|psi| psi.some.avg10)
    };
    Features::default()
        .with(Metric::CpuBusy, cpu)
        .with(
            Metric::MemoryUsed,
            memory.map(|memory| {
                percent(
                    memory.total_kib - memory.available_kib.min(memory.total_kib),
                    memory.total_kib,
                )
            }),
        )
        .with(Metric::CpuPressure, stall(&sample.pressure.cpu))
        .with(Metric::MemoryPressure, stall(&sample.pressure.memory))
        .with(Metric::IoPressure, stall(&sample.pressure.io))
        .with(
            Metric::GpuBusy,
            gpu.and_then(|gpu| gpu.busy_percent).map(f32::from),
        )
        .with(
            Metric::VramUsed,
            gpu.and_then(|gpu| gpu.vram)
                .filter(|vram| vram.total > 0)
                .map(|vram| percent(vram.used, vram.total)),
        )
}

fn percent(part: u64, whole: u64) -> f32 {
    (part as f64 / whole as f64 * 100.0) as f32
}
