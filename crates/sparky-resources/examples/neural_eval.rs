use std::time::Instant;

use sparky_resources::application::encoding::{
    Example, HORIZONS, OUTPUTS, STEP_INPUTS, WINDOW, decode, examples,
};
use sparky_resources::domain::{Features, METRICS};
use sparky_resources::neural::{Adam, Mlp, SequenceModel};
use sparky_resources::predict::trace::parse;
use sparky_resources::predict::{Forecaster, Holt, Persistence};

const HIDDEN: usize = 32;
const EPOCHS: usize = 40;
const BATCH: usize = 16;
const SEED: u64 = 20261118;
const LEAK_GUARD: usize = 40;

fn main() -> std::io::Result<()> {
    let mut rows = Vec::new();
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path)?;
        rows.extend(text.lines().skip(1).filter_map(parse));
    }
    let all = examples(&rows);
    let split = all.len() * 7 / 10;
    let (train, test) = all.split_at(split);
    let train = &train[..train.len().saturating_sub(LEAK_GUARD)];
    let mut model = SequenceModel::new(STEP_INPUTS, HIDDEN, OUTPUTS, SEED);
    let mlp = Mlp::new(&[28, 32, 16, 10], SEED);
    println!(
        "amostras={} exemplos treino={} teste={}",
        rows.len(),
        train.len(),
        test.len()
    );
    println!(
        "parametros: gru={} mlp={} total={}",
        model.params.len(),
        mlp.params.len(),
        model.params.len() + mlp.params.len()
    );
    let started = Instant::now();
    let mut adam = Adam::new(0.003, model.params.len());
    for _ in 0..EPOCHS {
        for batch in train.chunks(BATCH) {
            for (window, targets) in batch {
                model.accumulate(window, targets);
            }
            adam.step(&mut model.params);
        }
    }
    let per_epoch = started.elapsed() / EPOCHS as u32;
    println!(
        "treino: {per_epoch:?} por epoca, {} passos",
        EPOCHS * train.len().div_ceil(BATCH)
    );
    compare(&rows, split, &model);
    latency(&model, &mlp, test);
    Ok(())
}

fn compare(rows: &[(f64, Features)], split: usize, model: &SequenceModel) {
    let mut errors = vec![[0.0_f64; 3]; OUTPUTS];
    let mut counts = [0_usize; OUTPUTS];
    let mut holts = [Holt::new(0.2, 0.0); METRICS.len()];
    let mut naive = [Persistence::default(); METRICS.len()];
    for (index, end) in (WINDOW..rows.len()).enumerate() {
        let (at, features) = &rows[end - 1];
        for (slot, metric) in METRICS.iter().enumerate() {
            if let Some(value) = features.get(*metric) {
                holts[slot].update(*at, value);
                naive[slot].update(*at, value);
            }
        }
        let Some((window, targets)) = examples(&rows[end - WINDOW..rows.len().min(end + 40)])
            .into_iter()
            .next()
        else {
            continue;
        };
        if index < split {
            continue;
        }
        let neural = decode(&model.predict(&window));
        for (output, target) in targets.iter().enumerate() {
            let Some(target) = target.map(|value| value * 100.0) else {
                continue;
            };
            let (slot, horizon) = (output % METRICS.len(), HORIZONS[output / METRICS.len()]);
            let predictions = [
                naive[slot].forecast(horizon),
                holts[slot].forecast(horizon),
                Some(neural[output]),
            ];
            for (method, predicted) in predictions.iter().enumerate() {
                errors[output][method] +=
                    f64::from((predicted.unwrap_or(0.0).clamp(0.0, 100.0) - target).abs());
            }
            counts[output] += 1;
        }
    }
    println!("metrica,horizonte_s,previsoes,mae_persistencia,mae_holt,mae_gru");
    for output in 0..OUTPUTS {
        let count = counts[output].max(1) as f64;
        if counts[output] == 0 {
            continue;
        }
        let [naive, holt, gru] = errors[output].map(|total| total / count);
        let (metric, horizon) = (
            METRICS[output % METRICS.len()],
            HORIZONS[output / METRICS.len()],
        );
        println!(
            "{metric:?},{horizon},{},{naive:.3},{holt:.3},{gru:.3}",
            counts[output]
        );
    }
}

fn latency(model: &SequenceModel, mlp: &Mlp, test: &[Example]) {
    let Some((window, _)) = test.first() else {
        return;
    };
    let input = vec![0.1; 28];
    let mut samples: Vec<u128> = (0..2000)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(model.predict(window));
            std::hint::black_box(mlp.predict(&input));
            start.elapsed().as_nanos()
        })
        .collect();
    samples.sort_unstable();
    println!(
        "inferencia gru+mlp: p50={} ns p95={} ns p99={} ns",
        samples[1000], samples[1900], samples[1980]
    );
}
