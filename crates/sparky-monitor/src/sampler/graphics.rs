use crate::domain::gpu::vendor;
use crate::domain::snapshot::GpuDevice;
use crate::ports::SystemFiles;

use super::digits;

pub fn gpus(files: &dyn SystemFiles) -> Vec<GpuDevice> {
    let mut cards: Vec<String> = files
        .list("/sys/class/drm")
        .into_iter()
        .filter(|name| name.strip_prefix("card").is_some_and(digits))
        .collect();
    cards.sort();
    cards
        .into_iter()
        .filter_map(|card| {
            let device = format!("/sys/class/drm/{card}/device");
            let number = |file: &str| {
                files
                    .read(&format!("{device}/{file}"))
                    .and_then(|text| text.trim().parse::<u64>().ok())
            };
            Some(GpuDevice {
                name: vendor(&files.read(&format!("{device}/vendor"))?).to_owned(),
                busy: number("gpu_busy_percent").map(|busy| busy as f32),
                vram_used: number("mem_info_vram_used"),
                vram_total: number("mem_info_vram_total"),
            })
        })
        .collect()
}
