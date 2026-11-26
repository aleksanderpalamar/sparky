use std::time::{Duration, Instant};

use sparky_resources::application::archive::unpack_samples;
use sparky_resources::application::learning::Plan;
use sparky_resources::application::replay::confirm::{Paired, confirm};
use sparky_resources::application::replay::holdout::{chain, freeze};
use sparky_resources::application::replay::report::summarize;
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

struct Run {
    plan: Plan,
    updates: usize,
    elapsed: Duration,
    finite: bool,
    pairs: Vec<Paired>,
}

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
    println!(
        "amostras={} treino={:?} reserva={:?} taxa={} semente={} epocas={}",
        rows.len(),
        holdout.training,
        holdout.reserved,
        base.rate,
        base.seed,
        base.epochs
    );
    println!(
        "decay,objetivo,passos_aplicados,restante_da_cabeca,duracao,finito,n,mae_rede,mae_estat,ganho_vs_atual [ic95]"
    );
    let mut current: Option<Vec<Paired>> = None;
    for objective in OBJECTIVES {
        for decay in DECAYS {
            let plan = Plan {
                decay,
                objective,
                ..base
            };
            let started = Instant::now();
            let frozen = freeze(plan, training);
            let run = Run {
                plan,
                updates: frozen.updates,
                elapsed: started.elapsed(),
                finite: frozen
                    .model
                    .as_ref()
                    .is_some_and(|model| model.params.is_finite()),
                pairs: confirm(&rows, holdout.reserved.clone(), frozen.model.as_ref())
                    .into_values()
                    .flatten()
                    .collect(),
            };
            let reference = current.get_or_insert_with(|| run.pairs.clone());
            report(&run, reference);
        }
    }
    Ok(())
}

fn report(run: &Run, reference: &[Paired]) {
    let summary = summarize(&run.pairs, SEED);
    let versus: Option<Vec<Paired>> = (reference.len() == run.pairs.len()).then(|| {
        reference
            .iter()
            .zip(&run.pairs)
            .filter_map(|(before, now)| {
                Some(Paired {
                    at: before.at,
                    neural: now.neural,
                    statistical: before.neural?,
                    persistence: None,
                })
            })
            .collect()
    });
    let gain = versus
        .and_then(|pairs| summarize(&pairs, SEED).gain)
        .map_or_else(
            || "-".to_owned(),
            |gain| format!("{:.4} [{:.4}; {:.4}]", gain.mean, gain.low, gain.high),
        );
    let neural = summary
        .neural
        .map_or_else(|| "-".to_owned(), |neural| format!("{neural:.4}"));
    println!(
        "{},{:?},{},{:.3e},{:.1?},{},{},{neural},{:.4},{gain}",
        run.plan.decay,
        run.plan.objective,
        run.updates,
        f64::from(1.0 - run.plan.decay).powi(i32::try_from(run.updates).unwrap_or(i32::MAX)),
        run.elapsed,
        run.finite,
        summary.count,
        summary.statistical,
    );
}
