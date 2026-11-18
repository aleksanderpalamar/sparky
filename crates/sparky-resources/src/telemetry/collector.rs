use std::path::{Path, PathBuf};

use sparky_hardware::pressure::{Pressure, parse_pressure};
use sparky_hardware::procfs::{available_memory_kib, swap_kib, total_memory_kib};
use sparky_hardware::stat::parse_stat;

use super::sample::{GpuSample, MemorySample, Pressures, SystemSample, fresh};
use crate::ports::Files;

const DRM: &str = "/sys/class/drm";

pub struct Collector<F> {
    files: F,
    gpu: Option<PathBuf>,
}

impl<F: Files> Collector<F> {
    pub fn new(files: F) -> Self {
        let gpu = discover_gpu(&files);
        Self { files, gpu }
    }

    pub fn files(&self) -> &F {
        &self.files
    }

    pub fn system(&self) -> SystemSample {
        let stat = self.text("/proc/stat").and_then(|text| parse_stat(&text));
        let (cpu, cores) = stat.map_or((None, Vec::new()), |(cpu, cores)| (Some(cpu), cores));
        SystemSample {
            cpu: fresh(cpu),
            cores,
            memory: fresh(self.text("/proc/meminfo").and_then(|text| memory(&text))),
            pressure: Pressures {
                cpu: fresh(self.pressure("cpu")),
                memory: fresh(self.pressure("memory")),
                io: fresh(self.pressure("io")),
            },
            gpu: fresh(self.gpu()),
        }
    }

    fn text(&self, path: impl AsRef<Path>) -> Option<String> {
        self.files.read(path.as_ref()).ok()
    }

    fn pressure(&self, resource: &str) -> Option<Pressure> {
        self.text(format!("/proc/pressure/{resource}"))
            .and_then(|text| parse_pressure(&text))
    }

    fn gpu(&self) -> Option<GpuSample> {
        let device = self.gpu.as_ref()?;
        let number = |name: &str| self.text(device.join(name))?.trim().parse::<u64>().ok();
        Some(GpuSample {
            busy_percent: u8::try_from(number("gpu_busy_percent")?.min(100)).ok()?,
            vram_used: number("mem_info_vram_used")?,
            vram_total: number("mem_info_vram_total")?,
        })
    }
}

fn memory(meminfo: &str) -> Option<MemorySample> {
    let (swap_total_kib, swap_free_kib) = swap_kib(meminfo).unwrap_or_default();
    Some(MemorySample {
        total_kib: total_memory_kib(meminfo)?,
        available_kib: available_memory_kib(meminfo)?,
        swap_total_kib,
        swap_free_kib,
    })
}

fn discover_gpu(files: &impl Files) -> Option<PathBuf> {
    let mut cards = files.entries(Path::new(DRM)).ok()?;
    cards.sort();
    cards
        .into_iter()
        .filter(|name| name.starts_with("card") && !name.contains('-'))
        .map(|card| Path::new(DRM).join(card).join("device"))
        .find(|device| files.read(&device.join("gpu_busy_percent")).is_ok())
}
