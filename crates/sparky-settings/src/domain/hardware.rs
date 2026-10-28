#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hardware {
    pub processor: Option<String>,
    pub gpus: Vec<String>,
    pub memory_kib: Option<u64>,
}

pub fn processor(cpuinfo: &str) -> Option<String> {
    ["model name", "Processor", "Hardware", "cpu model"]
        .into_iter()
        .find_map(|key| field(cpuinfo, key))
        .map(str::to_owned)
}

pub fn memory_kib(meminfo: &str) -> Option<u64> {
    let mut parts = field(meminfo, "MemTotal")?.split_whitespace();
    let kib = parts.next()?.parse::<u64>().ok()?;
    if kib == 0 || parts.next() != Some("kB") || parts.next().is_some() {
        return None;
    }
    Some(kib)
}

pub fn memory_label(kib: u64) -> String {
    format!("{:.1} GiB", kib as f64 / 1_048_576.0).replace('.', ",")
}

fn field<'a>(input: &'a str, key: &str) -> Option<&'a str> {
    input
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim())
        .find(|value| !value.is_empty())
}

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;
