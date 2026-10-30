const BINARY: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
const DECIMAL: [&str; 4] = ["B/s", "kB/s", "MB/s", "GB/s"];
const THRESHOLD: f64 = 1000.0;

pub fn bytes(value: u64) -> String {
    scaled(value as f64, 1024.0, &BINARY)
}

pub fn rate(bytes_per_second: f64) -> String {
    scaled(bytes_per_second.max(0.0), 1000.0, &DECIMAL)
}

pub fn percent(value: f32) -> String {
    format!("{}%", comma(format!("{value:.1}")))
}

pub fn frequency(megahertz: u32) -> String {
    match megahertz {
        0..1000 => format!("{megahertz} MHz"),
        _ => format!("{} GHz", comma(format!("{:.2}", megahertz as f64 / 1000.0))),
    }
}

pub fn celsius(value: f32) -> String {
    format!("{} °C", comma(format!("{value:.1}")))
}

fn scaled(value: f64, base: f64, units: &[&str]) -> String {
    let mut scaled = value;
    let mut unit = 0;
    while scaled >= THRESHOLD && unit + 1 < units.len() {
        scaled /= base;
        unit += 1;
    }
    match unit {
        0 => format!("{} {}", scaled.round() as u64, units[0]),
        _ => format!("{} {}", comma(format!("{scaled:.2}")), units[unit]),
    }
}

fn comma(text: String) -> String {
    text.replace('.', ",")
}
