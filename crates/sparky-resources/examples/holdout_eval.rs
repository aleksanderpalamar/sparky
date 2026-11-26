use std::time::Instant;

use sparky_resources::application::archive::unpack_samples;
use sparky_resources::application::drill::divide;
use sparky_resources::application::encoding::Sample;
use sparky_resources::application::learning::{Plan, Verdict};
use sparky_resources::application::progress::outcome;
use sparky_resources::application::replay::confirm::{Cuts, confirm};
use sparky_resources::application::replay::holdout::{chain, freeze};
use sparky_resources::application::replay::report::{line, summarize};
use sparky_resources::application::replay::sessions::{overlap, reserve, sessions};
use sparky_resources::application::validation::Errors;
use sparky_resources::neural::Hybrid;
use sparky_resources::persistence::checkpoint::decode;

const SHARE: f64 = 0.3;
const SEED: u64 = 20261126;

fn main() -> std::io::Result<()> {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!(
            "uso: holdout_eval <history.ckpt> [history.ckpt ...] (do mais antigo ao mais novo)"
        );
        std::process::exit(2);
    }
    let rows = load(&paths)?;
    let plan = Plan::default();
    println!(
        "plano: epocas={} lote={} taxa={} decay={} margem={} semente={}",
        plan.epochs, plan.batch, plan.rate, plan.decay, plan.margin, plan.seed
    );
    let Some(holdout) = reserve(&rows, SHARE) else {
        println!(
            "amostras={} insuficientes para reservar sessoes futuras",
            rows.len()
        );
        return Ok(());
    };
    let training = &rows[holdout.training.clone()];
    let inner = divide(training.len());
    println!(
        "amostras={} sessoes={} treino={:?} reserva={:?} validacao_interna={:?} sobreposicao_com_reserva={:.2}",
        rows.len(),
        sessions(&rows).len(),
        holdout.training,
        holdout.reserved,
        inner.split..training.len(),
        overlap(&(inner.split..training.len()), &holdout.reserved)
    );
    let started = Instant::now();
    let (model, verdict) = freeze(plan, training);
    println!("decisao do treino: {}", summary(&verdict));
    println!("treino congelado em {:?}", started.elapsed());
    let cuts = confirm(&rows, holdout.reserved.clone(), model.as_ref());
    publish(&cuts);
    gate(&cuts, &plan, model.as_ref());
    Ok(())
}

fn load(paths: &[String]) -> std::io::Result<Vec<Sample>> {
    let mut parts = Vec::new();
    for path in paths {
        match decode(&std::fs::read(path)?) {
            Ok(checkpoint) => parts.push(unpack_samples(&checkpoint.values, 0.0)),
            Err(_) => eprintln!("{path}: checkpoint invalido"),
        }
    }
    Ok(chain(parts))
}

fn publish(cuts: &Cuts) {
    println!(
        "corte,metrica,horizonte_s,n,mae_rede,mae_estat,mae_persistencia,ganho [ic95],margem_pct,alerta"
    );
    for ((segment, output), pairs) in cuts {
        println!("{}", line(*segment, *output, &summarize(pairs, SEED)));
    }
}

fn gate(cuts: &Cuts, plan: &Plan, model: Option<&Hybrid>) {
    let pairs: Vec<_> = cuts.values().flatten().copied().collect();
    let summary = summarize(&pairs, SEED);
    let errors = Errors {
        statistical: summary.statistical,
        neural: summary.neural,
        count: summary.count,
    };
    let finite = model.is_some_and(|model| model.params.is_finite());
    println!(
        "total: n={} rede={:?} estat={:.3} ganho={:?} -> gate de {:.0}% (inalterado) daria {:?}",
        summary.count,
        summary.neural,
        summary.statistical,
        summary.gain,
        plan.margin * 100.0,
        outcome(plan.margin, &errors, None, finite)
    );
}

fn summary(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Promoted(_, errors) => format!("aprovado {errors:?}"),
        Verdict::Rejected(errors, outcome) => format!("{outcome:?} {errors:?}"),
        other => format!("{other:?}"),
    }
}
