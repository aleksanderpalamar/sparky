use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrmClient {
    pub id: u64,
    pub engine_ns: u64,
    pub vram: u64,
}

pub fn parse_fdinfo(text: &str) -> Option<DrmClient> {
    let mut id = None;
    let mut engine_ns = 0;
    let mut resident = None;
    let mut memory = None;
    for (key, value) in text.lines().filter_map(|line| line.split_once(':')) {
        let value = value.trim();
        match key.trim() {
            "drm-client-id" => id = value.parse().ok(),
            "drm-resident-vram" | "drm-resident-local0" => resident = Some(amount(value)),
            "drm-memory-vram" => memory = Some(amount(value)),
            key if key.starts_with("drm-engine-") && !key.starts_with("drm-engine-capacity") => {
                engine_ns += amount(value)
            }
            _ => {}
        }
    }
    Some(DrmClient {
        id: id?,
        engine_ns,
        vram: resident.or(memory).unwrap_or_default(),
    })
}

pub fn clients(found: impl IntoIterator<Item = DrmClient>) -> Vec<DrmClient> {
    let unique: BTreeMap<u64, DrmClient> = found
        .into_iter()
        .map(|client| (client.id, client))
        .collect();
    unique.into_values().collect()
}

pub fn activity(previous: &[DrmClient], current: &[DrmClient]) -> u64 {
    current
        .iter()
        .map(|client| {
            let before = previous
                .iter()
                .find(|old| old.id == client.id)
                .map_or(0, |old| old.engine_ns);
            client.engine_ns.saturating_sub(before)
        })
        .sum()
}

pub fn vram(clients: &[DrmClient]) -> u64 {
    clients.iter().map(|client| client.vram).sum()
}

fn amount(value: &str) -> u64 {
    let mut parts = value.split_whitespace();
    let number = parts
        .next()
        .and_then(|number| number.parse::<u64>().ok())
        .unwrap_or_default();
    match parts.next() {
        Some("KiB") => number * 1024,
        Some("MiB") => number * 1024 * 1024,
        Some("GiB") => number * 1024 * 1024 * 1024,
        _ => number,
    }
}

pub fn vendor(id: &str) -> &'static str {
    match id.trim() {
        "0x1002" => "AMD",
        "0x8086" => "Intel",
        "0x10de" => "NVIDIA",
        _ => "GPU",
    }
}
