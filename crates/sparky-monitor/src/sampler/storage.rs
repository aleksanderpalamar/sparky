use crate::domain::disk::{Mount, owning_disk, parse_diskstats, parse_mounts, physical};
use crate::domain::snapshot::DiskSample;
use crate::ports::{Filesystems, SystemFiles};

use super::millidegrees;

pub fn disks(files: &dyn SystemFiles, filesystems: &dyn Filesystems) -> Vec<DiskSample> {
    let mut names: Vec<String> = files
        .list("/sys/block")
        .into_iter()
        .filter(|name| physical(name))
        .collect();
    names.sort();
    let counters = files
        .read("/proc/diskstats")
        .map(|text| parse_diskstats(&text, &names))
        .unwrap_or_default();
    let mounts = files
        .read("/proc/mounts")
        .map(|text| parse_mounts(&text))
        .unwrap_or_default();
    counters
        .into_iter()
        .map(|counters| {
            let owned: Vec<&Mount> = mounts
                .iter()
                .filter(|mount| disk_of(files, &mount.device, &names) == Some(&counters.name))
                .collect();
            let (capacity, used) = owned
                .iter()
                .filter_map(|mount| filesystems.usage(&mount.path))
                .fold((0, 0), |(capacity, used), (total, taken)| {
                    (capacity + total, used + taken)
                });
            DiskSample {
                model: model(files, &counters.name),
                temperature: temperature(files, &counters.name),
                mounts: owned.iter().map(|mount| mount.path.clone()).collect(),
                capacity,
                used,
                counters,
            }
        })
        .collect()
}

fn disk_of<'a>(files: &dyn SystemFiles, device: &str, disks: &'a [String]) -> Option<&'a String> {
    let name = device.rsplit('/').next()?;
    let mapped = files
        .link(device)
        .and_then(|target| target.rsplit('/').next().map(str::to_owned))
        .filter(|target| target.starts_with("dm-"));
    let partition = match mapped {
        Some(dm) => files
            .list(&format!("/sys/block/{dm}/slaves"))
            .into_iter()
            .next()?,
        None => name.to_owned(),
    };
    owning_disk(&partition, disks)
}

fn model(files: &dyn SystemFiles, disk: &str) -> Option<String> {
    files
        .read(&format!("/sys/block/{disk}/device/model"))
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn temperature(files: &dyn SystemFiles, disk: &str) -> Option<f32> {
    let device = format!("/sys/block/{disk}/device");
    let hwmon = files
        .list(&device)
        .into_iter()
        .find(|entry| entry.starts_with("hwmon"))?;
    millidegrees(files.read(&format!("{device}/{hwmon}/temp1_input"))?)
}
