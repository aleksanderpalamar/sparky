use std::fs;

#[derive(Debug)]
pub struct Memory {
    pub rss_kib: Option<u64>,
    pub peak_kib: Option<u64>,
    pub anonymous_kib: Option<u64>,
    pub swap_kib: Option<u64>,
    pub available_kib: Option<u64>,
    pub swap_free_kib: Option<u64>,
    pub threads: Option<u64>,
    pub pressure_some_avg10: Option<f64>,
    pub pressure_full_avg10: Option<f64>,
}

impl Memory {
    pub fn read() -> Self {
        let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
        let memory = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let pressure = fs::read_to_string("/proc/pressure/memory").unwrap_or_default();
        Self::parse(&status, &memory, &pressure)
    }

    pub fn parse(status: &str, memory: &str, pressure: &str) -> Self {
        Self {
            rss_kib: value(status, "VmRSS"),
            peak_kib: value(status, "VmHWM"),
            anonymous_kib: value(status, "RssAnon"),
            swap_kib: value(status, "VmSwap"),
            available_kib: value(memory, "MemAvailable"),
            swap_free_kib: value(memory, "SwapFree"),
            threads: value(status, "Threads"),
            pressure_some_avg10: pressure_average(pressure, "some"),
            pressure_full_avg10: pressure_average(pressure, "full"),
        }
    }
}

fn pressure_average(text: &str, field: &str) -> Option<f64> {
    let line = text
        .lines()
        .find(|line| line.split_whitespace().next() == Some(field))?;
    line.split_whitespace()
        .find_map(|part| part.strip_prefix("avg10=")?.parse().ok())
}

fn value(text: &str, field: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name == field)
            .then_some(value)?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

impl std::fmt::Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rss_kib={:?} peak_kib={:?} anonymous_kib={:?} swap_kib={:?} available_kib={:?} swap_free_kib={:?} threads={:?} pressure_some_avg10={:?} pressure_full_avg10={:?}",
            self.rss_kib,
            self.peak_kib,
            self.anonymous_kib,
            self.swap_kib,
            self.available_kib,
            self.swap_free_kib,
            self.threads,
            self.pressure_some_avg10,
            self.pressure_full_avg10
        )
    }
}
