#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskCounters {
    pub name: String,
    pub read: u64,
    pub written: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub device: String,
    pub path: String,
}

const VIRTUAL: [&str; 7] = ["loop", "ram", "zram", "dm-", "md", "sr", "fd"];
const SECTOR: u64 = 512;

pub fn physical(name: &str) -> bool {
    !VIRTUAL.iter().any(|prefix| name.starts_with(prefix))
}

pub fn parse_diskstats(text: &str, disks: &[String]) -> Vec<DiskCounters> {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let name = *fields.get(2)?;
            if !disks.iter().any(|disk| disk == name) {
                return None;
            }
            Some(DiskCounters {
                name: name.to_owned(),
                read: fields.get(5)?.parse::<u64>().ok()? * SECTOR,
                written: fields.get(9)?.parse::<u64>().ok()? * SECTOR,
            })
        })
        .collect()
}

pub fn parse_mounts(text: &str) -> Vec<Mount> {
    let mut mounts: Vec<Mount> = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let (Some(device), Some(path)) = (fields.next(), fields.next()) else {
            continue;
        };
        let real = device.starts_with("/dev/") && !device.starts_with("/dev/loop");
        if real && !mounts.iter().any(|mount| mount.device == device) {
            mounts.push(Mount {
                device: device.to_owned(),
                path: path.replace("\\040", " "),
            });
        }
    }
    mounts
}

pub fn owning_disk<'a>(partition: &str, disks: &'a [String]) -> Option<&'a String> {
    disks
        .iter()
        .filter(|disk| partition.starts_with(disk.as_str()))
        .max_by_key(|disk| disk.len())
}
