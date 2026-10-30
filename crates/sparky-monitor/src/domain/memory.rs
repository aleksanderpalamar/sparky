#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Memory {
    pub total: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl Memory {
    pub fn parse(meminfo: &str) -> Option<Self> {
        let field = |key: &str| {
            meminfo
                .lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix(':'))
                .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
                .map(|kib| kib * 1024)
        };
        Some(Self {
            total: field("MemTotal")?,
            available: field("MemAvailable").unwrap_or_default(),
            swap_total: field("SwapTotal").unwrap_or_default(),
            swap_free: field("SwapFree").unwrap_or_default(),
        })
    }

    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    pub fn percent(&self) -> f32 {
        match self.total {
            0 => 0.0,
            total => (self.used() as f64 / total as f64 * 100.0) as f32,
        }
    }

    pub fn swap_used(&self) -> u64 {
        self.swap_total.saturating_sub(self.swap_free)
    }
}
