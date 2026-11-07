const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
const STEP: f64 = 1024.0;
const MINUTE: u64 = 60;
const HOUR: u64 = 60;
const DAY: u64 = 24;

pub fn bytes(amount: u64) -> String {
    let mut value = amount as f64;
    let mut unit = UNITS[0];
    for next in &UNITS[1..] {
        if value < STEP {
            break;
        }
        value /= STEP;
        unit = next;
    }
    format!("{value:.1} {unit}")
}

pub fn usage(used: u64, total: u64) -> Option<String> {
    if total == 0 {
        return None;
    }
    let percent = (used as f64 * 100.0 / total as f64).round();
    Some(format!("{} / {} ({percent}%)", bytes(used), bytes(total)))
}

pub fn duration(seconds: u64) -> String {
    let minutes = seconds / MINUTE;
    let hours = minutes / HOUR;
    let days = hours / DAY;
    let (hours, minutes) = (hours % DAY, minutes % HOUR);
    match (days, hours) {
        (0, 0) => format!("{minutes}m"),
        (0, _) => format!("{hours}h {minutes}m"),
        _ => format!("{days}d {hours}h {minutes}m"),
    }
}
