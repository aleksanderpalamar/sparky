use crate::neural::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub mean: f64,
    pub low: f64,
    pub high: f64,
}

pub fn interval(values: &[f64], block: usize, draws: usize, seed: u64) -> Option<Interval> {
    if values.is_empty() || draws == 0 {
        return None;
    }
    let length = values.len();
    let block = block.clamp(1, length);
    let starts = (length - block + 1) as u64;
    let mut rng = Rng::seeded(seed);
    let mut means: Vec<f64> = (0..draws)
        .map(|_| {
            let mut drawn = Vec::with_capacity(length);
            while drawn.len() < length {
                let start = (rng.next_u64() % starts) as usize;
                let room = length - drawn.len();
                drawn.extend(values[start..start + block].iter().take(room));
            }
            mean(&drawn)
        })
        .collect();
    means.sort_by(f64::total_cmp);
    let at = |fraction: f64| means[((draws - 1) as f64 * fraction).round() as usize];
    Some(Interval {
        mean: mean(values),
        low: at(0.025),
        high: at(0.975),
    })
}

pub fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len().max(1) as f64
}
