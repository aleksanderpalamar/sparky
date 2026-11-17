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

fn merge(kept: &mut Record, other: Record) {
    let paired = kept.device.paired || other.device.paired;
    let connected = kept.device.connected || other.device.connected;
    if preferred(&other, kept) {
        *kept = other;
    }
    kept.device.paired = paired;
    kept.device.connected = connected;
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
