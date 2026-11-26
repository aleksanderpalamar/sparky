use std::time::Instant;

use sparky_resources::application::archive::unpack_samples;
use sparky_resources::application::drill::divide;
use sparky_resources::application::encoding::WINDOW;
use sparky_resources::application::learning::Plan;
use sparky_resources::application::replay::confirm::{Paired, confirm};
use sparky_resources::application::replay::holdout::{chain, freeze};
use sparky_resources::application::replay::resample::interval;
use sparky_resources::application::replay::sessions::reserve;
use sparky_resources::neural::objective::Objective;
use sparky_resources::persistence::checkpoint::decode;

const SHARE: f64 = 0.3;
const SEED: u64 = 20261126;
const DECAYS: [f32; 4] = [0.1, 0.03, 0.01, 0.001];
const OBJECTIVES: [Objective; 3] = [
    Objective::Squared,
    Objective::Huber(0.02),
    Objective::Absolute,
];

fn main() -> std::io::Result<()> {
    let mut parts = Vec::new();
    for path in std::env::args().skip(1) {
        if let Ok(checkpoint) = decode(&std::fs::read(&path)?) {
            parts.push(unpack_samples(&checkpoint.values, 0.0));
        }
    }
    let rows = chain(parts);
    let Some(holdout) = reserve(&rows, SHARE) else {
        eprintln!("uso: decay_eval <history.ckpt> [...] com coleta suficiente para reservar");
        std::process::exit(2);
    };
    let training = &rows[holdout.training.clone()];
    let base = Plan::default();
    let steps = steps(&base, training.len());
    println!(
        "amostras={} treino={:?} reserva={:?} taxa={} semente={} epocas={} passos<={steps}",
        rows.len(),
        holdout.training,
        holdout.reserved,
        base.rate,
        base.seed,
        base.epochs
    );
    println!(
        "decay,objetivo,restante_apos_passos,duracao,finito,n,mae_rede,mae_estat,ganho_vs_atual [ic95]"
    );
    let mut reference: Option<Vec<f64>> = None;
    for objective in OBJECTIVES {
        for decay in DECAYS {
            let plan = Plan {
                decay,
                objective,
                ..base
            };
            let started = Instant::now();
            let (model, _) = freeze(plan, training);
            let elapsed = started.elapsed();
            let pairs: Vec<Paired> = confirm(&rows, holdout.reserved.clone(), model.as_ref())
                .into_values()
                .flatten()
                .collect();
            let errors: Vec<f64> = pairs.iter().filter_map(|pair| pair.neural).collect();
            let current = reference.get_or_insert_with(|| errors.clone());
            report(
                &plan,
                steps,
                elapsed,
                model.is_some_and(|model| model.params.is_finite()),
                &pairs,
                &errors,
                current,
            );
        }
    }
    Ok(())
}

fn steps(plan: &Plan, rows: usize) -> usize {
    let division = divide(rows);
    plan.epochs * division.last.saturating_sub(WINDOW).div_ceil(plan.batch)
}

fn report(
    plan: &Plan,
    steps: usize,
    elapsed: std::time::Duration,
    finite: bool,
    pairs: &[Paired],
    errors: &[f64],
    current: &[f64],
) {
    let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len().max(1) as f64;
    let statistical: Vec<f64> = pairs.iter().map(|pair| pair.statistical).collect();
    let paired: Vec<f64> = current
        .iter()
        .zip(errors)
        .map(|(before, now)| before - now)
        .collect();
    let gain = (paired.len() == current.len())
        .then(|| interval(&paired, 30, 1000, SEED))
        .flatten()
        .map_or_else(
            || "-".to_owned(),
            |gain| format!("{:.4} [{:.4}; {:.4}]", gain.mean, gain.low, gain.high),
        );
    println!(
        "{},{:?},{:.3e},{elapsed:.1?},{finite},{},{:.4},{:.4},{gain}",
        plan.decay,
        plan.objective,
        f64::from(1.0 - plan.decay).powi(steps as i32),
        errors.len(),
        mean(errors),
        mean(&statistical),
    );
}
