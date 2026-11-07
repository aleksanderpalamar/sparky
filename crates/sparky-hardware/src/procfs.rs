pub fn processor(cpuinfo: &str) -> Option<String> {
    ["model name", "Processor", "Hardware", "cpu model"]
        .into_iter()
        .find_map(|key| field(cpuinfo, key))
        .map(str::to_owned)
}

pub fn threads(cpuinfo: &str) -> usize {
    entries(cpuinfo, "processor")
        .filter(|value| value.parse::<usize>().is_ok())
        .count()
}

pub fn total_memory_kib(meminfo: &str) -> Option<u64> {
    kib(meminfo, "MemTotal").filter(|kib| *kib > 0)
}

pub fn available_memory_kib(meminfo: &str) -> Option<u64> {
    kib(meminfo, "MemAvailable")
}

fn kib(meminfo: &str, key: &str) -> Option<u64> {
    let mut parts = field(meminfo, key)?.split_whitespace();
    let kib = parts.next()?.parse::<u64>().ok()?;
    if parts.next() != Some("kB") || parts.next().is_some() {
        return None;
    }
    Some(kib)
}

fn field<'a>(input: &'a str, key: &str) -> Option<&'a str> {
    entries(input, key).find(|value| !value.is_empty())
}

fn entries<'a>(input: &'a str, key: &str) -> impl Iterator<Item = &'a str> {
    input
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(move |(name, _)| name.trim() == key)
        .map(|(_, value)| value.trim())
}

#[cfg(test)]
#[path = "procfs_tests.rs"]
mod tests;
