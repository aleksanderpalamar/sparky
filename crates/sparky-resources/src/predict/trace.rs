use crate::domain::{Features, METRICS, Metric};

pub const HEADER: &str =
    "at,cpu_busy,memory_used,cpu_pressure,memory_pressure,io_pressure,gpu_busy,vram_used";

pub fn line(at: f64, features: &Features) -> String {
    let values = METRICS.iter().map(|metric| {
        features
            .get(*metric)
            .map_or(String::new(), |value| format!("{value:.3}"))
    });
    std::iter::once(format!("{at:.3}"))
        .chain(values)
        .collect::<Vec<_>>()
        .join(",")
}

pub fn parse(line: &str) -> Option<(f64, Features)> {
    let mut fields = line.trim().split(',');
    let at = fields
        .next()?
        .parse::<f64>()
        .ok()
        .filter(|at| at.is_finite())?;
    let mut features = Features::default();
    for metric in METRICS {
        features = features.with(metric, value(fields.next()?)?);
    }
    fields.next().is_none().then_some((at, features))
}

pub fn series(rows: &[(f64, Features)], metric: Metric) -> Vec<super::Point> {
    rows.iter()
        .filter_map(|(at, features)| {
            features
                .get(metric)
                .map(|value| super::Point { at: *at, value })
        })
        .collect()
}

fn value(field: &str) -> Option<Option<f32>> {
    if field.is_empty() {
        return Some(None);
    }
    field.parse::<f32>().ok().map(Some)
}
