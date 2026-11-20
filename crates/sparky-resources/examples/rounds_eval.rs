use std::time::{Duration, Instant};

use sparky_resources::application::archive::unpack_samples;
use sparky_resources::application::diagnosis::{SEGMENTS, Segment, Tally};
use sparky_resources::application::encoding::OUTPUTS;
use sparky_resources::application::learning::{Learner, Plan, Verdict};
use sparky_resources::persistence::checkpoint::decode;

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let Some(path) = arguments.next() else {
        eprintln!("uso: rounds_eval <history.ckpt> [rodadas] [decay] [rate]");
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
        "amostras={} decay={} rate={}",
        samples.len(),
        plan.decay,
        plan.rate
    );
    let start = samples.len() / 2;
    let chunk = (samples.len() - start) / rounds.max(1);
    let mut arriving = samples.split_off(start).into_iter();
    let mut learner = Learner::new(plan);
    learner.resume(samples, None);
    for round in 1..=rounds {
        for sample in arriving.by_ref().take(chunk) {
            learner.record(sample);
        }
        let verdict = finish(&mut learner);
        let summary = match &verdict {
            Verdict::Promoted(_, errors) => format!("aprovado {errors:?}"),
            Verdict::Rejected(errors, outcome) => format!("{outcome:?} {errors:?}"),
            other => format!("{other:?}"),
        };
        println!("rodada {round}: {summary}");
        if let Some(tally) = learner.diagnosis() {
            report(tally);
        }
    }
    Ok(())
}

fn finish(learner: &mut Learner) -> Verdict {
    loop {
        match learner.step(None, Instant::now() + Duration::from_millis(200)) {
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
