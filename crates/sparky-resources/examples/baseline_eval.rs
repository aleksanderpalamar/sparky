use std::time::Instant;

use sparky_resources::domain::METRICS;
use sparky_resources::predict::trace::{parse, series};
use sparky_resources::predict::{Holt, Persistence, evaluate};

const HORIZONS: [f64; 2] = [10.0, 60.0];

fn main() -> std::io::Result<()> {
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path)?;
        let rows: Vec<_> = text.lines().skip(1).filter_map(parse).collect();
        println!("# {path}: {} amostras", rows.len());
        println!(
            "metrica,horizonte_s,previsoes,mae_persistencia,mae_holt_0.5_0.2,mae_ses_0.5,mae_ses_0.2,mae_holt_0.3_0.02"
        );
        for metric in METRICS {
            let points = series(&rows, metric);
            for horizon in HORIZONS {
                let naive = evaluate(&points, &mut Persistence::default(), horizon);
                let others = [(0.5, 0.2), (0.5, 0.0), (0.2, 0.0), (0.3, 0.02)]
                    .map(|(alpha, beta)| evaluate(&points, &mut Holt::new(alpha, beta), horizon));
                if let Some(naive) = naive {
                    let errors: Vec<String> = others
                        .iter()
                        .map(|score| {
                            score.map_or("-".to_owned(), |score| {
                                format!("{:.3}", score.mean_absolute_error)
                            })
                        })
                        .collect();
                    println!(
                        "{metric:?},{horizon},{},{:.3},{}",
                        naive.forecasts,
                        naive.mean_absolute_error,
                        errors.join(",")
                    );
                }
            }
        }
        latency(&rows);
    }
    Ok(())
}

fn latency(rows: &[(f64, sparky_resources::domain::Features)]) {
    let mut baseline = sparky_resources::application::Baseline::default();
    let mut samples: Vec<u128> = rows
        .iter()
        .map(|(at, features)| {
            let start = Instant::now();
            baseline.observe(*at, features);
            start.elapsed().as_nanos()
        })
        .collect();
    samples.sort_unstable();
    if let (Some(median), Some(high)) = (
        samples.get(samples.len() / 2),
        samples.get(samples.len() * 95 / 100),
    ) {
        println!("latencia do baseline por amostra: p50={median} ns p95={high} ns");
    }
}
