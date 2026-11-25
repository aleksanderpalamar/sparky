use std::time::{Duration, Instant};

use sparky_resources::application::archive::unpack_samples;
use sparky_resources::application::diagnosis::{SEGMENTS, Segment, Tally};
use sparky_resources::application::encoding::OUTPUTS;
use sparky_resources::application::learning::shape;
use sparky_resources::application::learning::{Learner, Plan, Verdict};
use sparky_resources::domain::activity::ACTIVITIES;
use sparky_resources::neural::{Hybrid, Shape};
use sparky_resources::persistence::checkpoint::decode;

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let Some(path) = arguments.next() else {
        eprintln!(
            "uso: rounds_eval <history.ckpt> [rodadas] [decay] [rate] [candidato.ckpt] [zerar]"
        );
        std::process::exit(2);
    };
    let rounds: usize = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(6);
    let defaults = Plan::default();
    let plan = Plan {
        decay: arguments
            .next()
            .and_then(|text| text.parse().ok())
            .unwrap_or(defaults.decay),
        rate: arguments
            .next()
            .and_then(|text| text.parse().ok())
            .unwrap_or(defaults.rate),
        ..defaults
    };
    let Ok(checkpoint) = decode(&std::fs::read(path)?) else {
        eprintln!("checkpoint inválido");
        std::process::exit(1);
    };
    let mut samples = unpack_samples(&checkpoint.values, 0.0);
    println!(
        "amostras={} plano: epocas={} lote={} taxa={} decay={} margem={} semente={}",
        samples.len(),
        plan.epochs,
        plan.batch,
        plan.rate,
        plan.decay,
        plan.margin,
        plan.seed
    );
    let start = samples.len() / 2;
    let chunk = (samples.len() - start) / rounds.max(1);
    let mut arriving = samples.split_off(start).into_iter();
    let inherited = arguments.next().and_then(|path| migrated(&path, &plan));
    let inherited = match (inherited, arguments.next().as_deref()) {
        (Some(mut model), Some("zerar")) => {
            model.settle(1.0);
            Some(model)
        }
        (model, _) => model,
    };
    println!("candidato herdado: {}", inherited.is_some());
    let mut learner = Learner::new(plan);
    learner.resume(samples, inherited);
    let mut active: Option<Hybrid> = None;
    for round in 1..=rounds {
        for sample in arriving.by_ref().take(chunk) {
            learner.record(sample);
        }
        let verdict = finish(&mut learner, active.as_ref());
        if let Verdict::Promoted(model, _) = &verdict {
            active = Some((**model).clone());
        }
        let summary = match &verdict {
            Verdict::Promoted(_, errors) => format!("aprovado {errors:?}"),
            Verdict::Rejected(errors, outcome) => format!("{outcome:?} {errors:?}"),
            other => format!("{other:?}"),
        };
        println!("rodada {round} (ativo: {}): {summary}", active.is_some());
        if let Some(tally) = learner.diagnosis() {
            report(tally);
        }
    }
    Ok(())
}

fn finish(learner: &mut Learner, active: Option<&Hybrid>) -> Verdict {
    loop {
        match learner.step(active, Instant::now() + Duration::from_millis(200)) {
            Verdict::Training => continue,
            other => return other,
        }
    }
}

fn report(tally: &Tally) {
    for segment in SEGMENTS {
        let (count, neural, statistical) = totals(tally, segment);
        if count == 0 {
            continue;
        }
        let cpu = tally.cell(segment, 0);
        println!(
            "  {:8} n={count:5} rede={:.2} estat={:.2} | cpu10s rede={:.2} estat={:.2}",
            segment.label(),
            neural / count as f64,
            statistical / count as f64,
            cpu.neural / cpu.count.max(1) as f64,
            cpu.statistical / cpu.count.max(1) as f64,
        );
    }
}

fn totals(tally: &Tally, segment: Segment) -> (usize, f64, f64) {
    (0..OUTPUTS).map(|output| tally.cell(segment, output)).fold(
        (0, 0.0, 0.0),
        |(count, neural, statistical), cell| {
            (
                count + cell.count,
                neural + cell.neural,
                statistical + cell.statistical,
            )
        },
    )
}

fn migrated(path: &str, plan: &Plan) -> Option<Hybrid> {
    let checkpoint = decode(&std::fs::read(path).ok()?).ok()?;
    let legacy = Shape {
        context: ACTIVITIES.len(),
        ..shape(plan)
    };
    let mut model = Hybrid::new(legacy, 0);
    if model.params.len() != checkpoint.values.len() {
        return None;
    }
    model.params.values = checkpoint.values;
    model.widen(shape(plan))
}
