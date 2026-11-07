#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hardware {
    pub processor: Option<String>,
    pub gpus: Vec<String>,
    pub memory_kib: Option<u64>,
}

pub fn memory_label(kib: u64) -> String {
    format!("{:.1} GiB", kib as f64 / 1_048_576.0).replace('.', ",")
}

#[cfg(test)]
#[path = "hardware_tests.rs"]
mod tests;
