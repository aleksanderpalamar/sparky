use std::time::{Duration, Instant};

use sparky_resources::application::drill::{Division, divide};
use sparky_resources::application::encoding::{
    Example, HORIZONS, OUTPUTS, Sample, WINDOW, absolute, decode, example_at, examples,
};
use sparky_resources::application::learning::{Learner, Plan, Verdict, lesson};
use sparky_resources::application::reference::bases;
use sparky_resources::domain::{Activities, METRICS};
use sparky_resources::neural::Hybrid;
use sparky_resources::predict::trace::parse;
use sparky_resources::predict::{Forecaster, Holt, Persistence};

fn main() -> std::io::Result<()> {
    let mut rows = Vec::new();
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path)?;
        rows.extend(
            text.lines()
                .skip(1)
                .filter_map(parse)
                .map(|(at, features)| Sample {
                    at,
                    features,
                    activities: Activities::default(),
                }),
        );
    }
    let plan = Plan::default();
    let kept = rows.len().saturating_sub(plan.capacity);
    let rows = rows.split_off(kept);
    let division = divide(rows.len());
    println!(
        "plano: epocas={} lote={} taxa={} decay={} margem={} semente={}",
        plan.epochs, plan.batch, plan.rate, plan.decay, plan.margin, plan.seed
    );
    coverage(&rows, division);
    let started = Instant::now();
    let mut learner = Learner::new(plan);
    learner.resume(rows.clone(), None);
    let verdict = finish(&mut learner);
    println!(
        "treino: {:?}, decisao: {}",
        started.elapsed(),
        summary(&verdict)
    );
    let model = match verdict {
        Verdict::Promoted(model, _) => Some(*model),
        _ => learner.incumbent().cloned(),
    };
    let Some(model) = model else {
        println!("sem modelo para avaliar");
        return Ok(());
    };
    println!("parametros: gru+mlp={}", model.params.len());
    compare(&rows, division.split, &model);
    latency(&model, &examples(&rows[division.split..]));
    Ok(())
}

fn coverage(rows: &[Sample], division: Division) {
    let bases = bases(rows);
    let training = (WINDOW..division.last)
        .filter(|end| lesson(rows, &bases, division.split, *end).is_some())
        .count();
    let evaluation = (division.split.max(WINDOW)..=rows.len())
        .filter(|end| example_at(rows, &bases, *end).is_some())
        .count();
    println!(
        "amostras={} corte={} exemplos treino={training} avaliacao={evaluation}",
        rows.len(),
        division.split
    );
}

fn finish(learner: &mut Learner) -> Verdict {
    loop {
        match learner.step(None, Instant::now() + Duration::from_millis(200)) {
            Verdict::Training => continue,
            other => return other,
        }
    }
}

fn summary(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Promoted(_, errors) => format!("aprovado {errors:?}"),
        Verdict::Rejected(errors, outcome) => format!("{outcome:?} {errors:?}"),
        other => format!("{other:?}"),
    }
}

fn compare(rows: &[Sample], split: usize, model: &Hybrid) {
    let mut errors = vec![[0.0_f64; 3]; OUTPUTS];
    let mut counts = [0_usize; OUTPUTS];
    let mut holts = [Holt::new(0.2, 0.0); METRICS.len()];
    let mut naive = [Persistence::default(); METRICS.len()];
    let bases = bases(rows);
    for end in WINDOW..rows.len() {
        let now = &rows[end - 1];
        for (slot, metric) in METRICS.iter().enumerate() {
            if let Some(value) = now.features.get(*metric) {
                holts[slot].update(now.at, value);
                naive[slot].update(now.at, value);
            }
        }
        let (Some(example), Some(base)) = (example_at(rows, &bases, end), bases.get(end - 1))
        else {
            continue;
        };
        if end - 1 < split {
            continue;
        }
        let neural = decode(&model.predict(&example.window, &example.context), base);
        for (output, target) in absolute(&example.targets, base).iter().enumerate() {
            let Some(target) = *target else {
                continue;
            };
            let (slot, horizon) = (output % METRICS.len(), HORIZONS[output / METRICS.len()]);
            let predictions = [
                naive[slot].forecast(horizon),
                holts[slot].forecast(horizon),
                neural[output],
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

fn latency(model: &Hybrid, test: &[Example]) {
    let Some(example) = test.first() else {
        return;
    };
    let mut samples: Vec<u128> = (0..2000)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(model.predict(&example.window, &example.context));
            start.elapsed().as_nanos()
        })
        .collect();
    samples.sort_unstable();
    println!(
        "inferencia gru+mlp: p50={} ns p95={} ns p99={} ns",
        samples[1000], samples[1900], samples[1980]
    );
}
