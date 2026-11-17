use super::bluetooth::Device;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub device: Device,
    pub address: Option<String>,
}

pub fn physical(records: Vec<Record>) -> Vec<Device> {
    let mut chosen: Vec<Record> = Vec::new();
    for record in records {
        match chosen.iter_mut().find(|kept| same_device(kept, &record)) {
            Some(kept) => merge(kept, record),
            None => chosen.push(record),
        }
    }
    chosen.into_iter().map(|record| record.device).collect()
}

fn same_device(first: &Record, second: &Record) -> bool {
    match (&first.address, &second.address) {
        (Some(first), Some(second)) => first.eq_ignore_ascii_case(second),
        _ => false,
    }
}

fn merge(kept: &mut Record, mut other: Record) {
    if preferred(&other, kept) {
        std::mem::swap(kept, &mut other);
    }
    kept.device.paired |= other.device.paired;
    kept.device.connected |= other.device.connected;
    kept.device.duplicates.push(other.device.path);
    kept.device.duplicates.append(&mut other.device.duplicates);
    kept.device.duplicates.sort();
}

fn preferred(candidate: &Record, kept: &Record) -> bool {
    match rank(candidate).cmp(&rank(kept)) {
        std::cmp::Ordering::Equal => candidate.device.path < kept.device.path,
        ordering => ordering.is_gt(),
    }
}

fn rank(record: &Record) -> (bool, bool, bool) {
    (
        record.device.connected,
        named_by_address(record),
        record.device.paired,
    )
}

fn named_by_address(record: &Record) -> bool {
    record.address.as_deref().is_some_and(|address| {
        let suffix = format!("dev_{}", address.replace(':', "_"));
        record
            .device
            .path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(&suffix))
    })
}
