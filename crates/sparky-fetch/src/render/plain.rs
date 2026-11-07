use crate::domain::report::Report;
use crate::render::{label, shown};

pub fn render(report: &Report) -> String {
    report
        .entries
        .iter()
        .map(|entry| format!("{}: {}\n", label(entry.field), shown(&entry.status)))
        .collect()
}
