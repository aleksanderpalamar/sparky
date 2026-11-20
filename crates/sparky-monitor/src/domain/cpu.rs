pub use sparky_hardware::stat::{CpuTimes, parse_stat, usage};

pub fn model(cpuinfo: &str) -> Option<String> {
    cpuinfo
        .lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split_once(':'))
        .map(|(_, name)| name.trim().to_owned())
        .filter(|name| !name.is_empty())
}

pub fn mean_frequency(kilohertz: &[u64]) -> Option<u32> {
    let count = kilohertz.len() as u64;
    let sum: u64 = kilohertz.iter().sum();
    sum.checked_div(count).map(|mean| (mean / 1000) as u32)
}

pub fn cpu_sensor(name: &str) -> bool {
    matches!(name, "k10temp" | "zenpower" | "coretemp" | "cpu_thermal")
}
