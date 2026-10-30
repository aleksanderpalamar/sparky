mod graphics;
mod processes;
mod storage;

#[cfg(test)]
pub(crate) mod fake_system;
#[cfg(test)]
mod sampler_tests;

use std::sync::Arc;
use std::time::Instant;

use crate::domain::cpu::{cpu_sensor, mean_frequency, model, parse_stat};
use crate::domain::memory::Memory;
use crate::domain::network::parse_net_dev;
use crate::domain::snapshot::{CpuInfo, Snapshot};
use crate::ports::{Filesystems, SystemFiles};

pub struct Sampler {
    files: Arc<dyn SystemFiles>,
    filesystems: Arc<dyn Filesystems>,
    page_size: u64,
}

impl Sampler {
    pub fn new(
        files: Arc<dyn SystemFiles>,
        filesystems: Arc<dyn Filesystems>,
        page_size: u64,
    ) -> Self {
        Self {
            files,
            filesystems,
            page_size,
        }
    }

    pub fn sample(&self, at: Instant) -> Snapshot {
        let files = self.files.as_ref();
        let (cpu, cores) = files
            .read("/proc/stat")
            .and_then(|text| parse_stat(&text))
            .unwrap_or_default();
        Snapshot {
            at,
            cpu,
            cores,
            cpu_info: self.cpu_info(),
            memory: files
                .read("/proc/meminfo")
                .and_then(|text| Memory::parse(&text)),
            disks: storage::disks(files, self.filesystems.as_ref()),
            networks: files
                .read("/proc/net/dev")
                .map(|text| parse_net_dev(&text))
                .unwrap_or_default(),
            gpus: graphics::gpus(files),
            processes: processes::processes(files, self.page_size),
        }
    }

    fn cpu_info(&self) -> CpuInfo {
        let files = self.files.as_ref();
        let frequencies: Vec<u64> = files
            .list("/sys/devices/system/cpu")
            .into_iter()
            .filter(|name| name.strip_prefix("cpu").is_some_and(digits))
            .filter_map(|cpu| {
                files.read(&format!(
                    "/sys/devices/system/cpu/{cpu}/cpufreq/scaling_cur_freq"
                ))
            })
            .filter_map(|text| text.trim().parse().ok())
            .collect();
        CpuInfo {
            model: files.read("/proc/cpuinfo").and_then(|text| model(&text)),
            frequency: mean_frequency(&frequencies),
            temperature: self.cpu_temperature(),
        }
    }

    fn cpu_temperature(&self) -> Option<f32> {
        let files = self.files.as_ref();
        files
            .list("/sys/class/hwmon")
            .into_iter()
            .find_map(|hwmon| {
                let base = format!("/sys/class/hwmon/{hwmon}");
                let name = files.read(&format!("{base}/name"))?;
                cpu_sensor(name.trim()).then_some(())?;
                millidegrees(files.read(&format!("{base}/temp1_input"))?)
            })
    }
}

pub(crate) fn digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

pub(crate) fn millidegrees(text: String) -> Option<f32> {
    text.trim().parse::<f32>().ok().map(|value| value / 1000.0)
}
