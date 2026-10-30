#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuTimes {
    pub busy: u64,
    pub total: u64,
}

pub fn parse_stat(text: &str) -> Option<(CpuTimes, Vec<CpuTimes>)> {
    let mut lines = text.lines().filter(|line| line.starts_with("cpu"));
    let total = times(lines.next()?)?;
    let cores = lines.filter_map(times).collect();
    Some((total, cores))
}

fn times(line: &str) -> Option<CpuTimes> {
    let fields = line
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let [user, nice, system, idle, iowait, irq, softirq, steal] = fields[..] else {
        return None;
    };
    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    Some(CpuTimes {
        busy: total - idle - iowait,
        total,
    })
}

pub fn usage(previous: CpuTimes, current: CpuTimes) -> f32 {
    let total = current.total.saturating_sub(previous.total);
    let busy = current.busy.saturating_sub(previous.busy);
    match total {
        0 => 0.0,
        _ => (busy as f64 / total as f64 * 100.0) as f32,
    }
}

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
