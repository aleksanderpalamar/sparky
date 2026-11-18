use std::time::{Duration, Instant};

use super::learning::{Learner, Plan, Verdict};
use crate::domain::{Features, Metric};

fn small() -> Plan {
    Plan {
        capacity: 400,
        minimum: 300,
        epochs: 30,
        batch: 16,
        rate: 0.01,
        margin: 0.05,
        hidden: 8,
        seed: 1,
    }
}

fn wave(learner: &mut Learner, count: usize) {
    for index in 0..count {
        let at = index as f64 * 2.0;
        let value = 50.0 + 30.0 * ((at / 40.0) * std::f64::consts::TAU).sin() as f32;
        learner.record(at, Features::default().with(Metric::CpuBusy, Some(value)));
    }
}

fn finish(learner: &mut Learner) -> Verdict {
    for _ in 0..10_000 {
        match learner.step(None, Instant::now() + Duration::from_millis(50)) {
            Verdict::Training => continue,
            other => return other,
        }
    }
    Verdict::Training
}

#[test]
fn espera_historico_suficiente_antes_de_treinar() {
    let mut learner = Learner::new(small());
    wave(&mut learner, 100);
    assert_eq!(
        learner.step(None, Instant::now() + Duration::from_secs(1)),
        Verdict::Waiting
    );
}

#[test]
fn treino_respeita_o_prazo_e_pode_ser_retomado() {
    let mut learner = Learner::new(small());
    wave(&mut learner, 400);
    assert_eq!(learner.step(None, Instant::now()), Verdict::Training);
    assert!(!matches!(
        finish(&mut learner),
        Verdict::Training | Verdict::Waiting
    ));
}

#[test]
fn padrao_periodico_promove_um_modelo_melhor_que_a_estatistica() {
    let mut learner = Learner::new(small());
    wave(&mut learner, 400);
    let Verdict::Promoted(model, errors) = finish(&mut learner) else {
        panic!("a GRU deveria vencer o Holt numa onda periódica");
    };
    assert!(
        errors
            .neural
            .is_some_and(|neural| neural < errors.statistical * 0.95),
        "{errors:?}"
    );
    assert!(model.params.is_finite());
}

#[test]
fn o_historico_e_limitado() {
    let mut learner = Learner::new(Plan {
        capacity: 10,
        minimum: 5,
        ..small()
    });
    wave(&mut learner, 50);
    assert_eq!(learner.step(None, Instant::now()), Verdict::Training);
}
