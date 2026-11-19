use std::time::{Duration, Instant};

use super::encoding::{OUTPUTS, STEP_INPUTS, Sample, WINDOW, example_at};
use super::learning::{Learner, Plan, Verdict, lesson};
use crate::domain::{Activities, Features, Metric};
use crate::neural::SequenceModel;

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

fn sample(at: f64, value: f32) -> Sample {
    Sample {
        at,
        features: Features::default().with(Metric::CpuBusy, Some(value)),
        activities: Activities::default(),
    }
}

fn wave(learner: &mut Learner, count: usize) {
    for index in 0..count {
        let at = index as f64 * 2.0;
        let value = 50.0 + 30.0 * ((at / 40.0) * std::f64::consts::TAU).sin() as f32;
        learner.record(sample(at, value));
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

#[test]
fn o_progresso_mostra_a_coleta_e_a_passada_atual() {
    let mut learner = Learner::new(small());
    wave(&mut learner, 120);
    let progress = learner.progress();
    assert_eq!((progress.collected, progress.needed), (120, 300));
    assert_eq!((progress.epoch, progress.epochs), (0, 30));
    assert!(progress.parameters > 0);
    wave(&mut learner, 400);
    while learner.progress().epoch == 0 {
        assert_eq!(
            learner.step(None, Instant::now() + Duration::from_millis(5)),
            Verdict::Training
        );
    }
    assert_eq!(learner.progress().collected, 400);
}

#[test]
fn cancelar_descarta_a_sessao_em_andamento() {
    let mut learner = Learner::new(small());
    wave(&mut learner, 400);
    while learner.progress().epoch == 0 {
        assert_eq!(
            learner.step(None, Instant::now() + Duration::from_millis(5)),
            Verdict::Training
        );
    }
    learner.cancel();
    assert_eq!(learner.progress().epoch, 0);
}

#[test]
fn o_treino_nunca_usa_alvos_da_parte_de_validacao() {
    let split = 420;
    let rows: Vec<Sample> = (0..600)
        .map(|index| sample(index as f64 * 0.25, if index < split { 10.0 } else { 90.0 }))
        .collect();
    let leaks = |targets: &[Option<f32>]| targets.iter().flatten().any(|target| *target > 0.5);
    assert!(example_at(&rows, 300).is_some_and(|(_, targets)| leaks(&targets)));
    for end in WINDOW..=split {
        assert!(
            !lesson(&rows, split, end).is_some_and(|(_, targets)| leaks(&targets)),
            "exemplo {end}"
        );
    }
}

fn fresh(plan: Plan) -> SequenceModel {
    SequenceModel::still(STEP_INPUTS, plan.hidden, OUTPUTS, plan.seed)
}

fn stubborn() -> Plan {
    Plan {
        margin: 1.0,
        epochs: 2,
        ..small()
    }
}

#[test]
fn rodada_recusada_continua_de_onde_parou() {
    let mut learner = Learner::new(stubborn());
    wave(&mut learner, 400);
    assert!(matches!(finish(&mut learner), Verdict::Rejected(..)));
    assert_eq!(learner.step(None, Instant::now()), Verdict::Training);
    assert!(
        learner
            .starting_point()
            .is_some_and(|model| *model != fresh(stubborn()))
    );
}

#[test]
fn cancelar_tambem_descarta_o_candidato_guardado() {
    let mut learner = Learner::new(stubborn());
    wave(&mut learner, 400);
    assert!(matches!(finish(&mut learner), Verdict::Rejected(..)));
    learner.cancel();
    assert_eq!(learner.step(None, Instant::now()), Verdict::Training);
    assert_eq!(learner.starting_point(), Some(&fresh(stubborn())));
}

#[test]
fn o_progresso_informa_a_janela_de_coleta() {
    let learner = Learner::new(small());
    assert_eq!(learner.progress().window, 400);
}
