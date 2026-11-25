use super::drill::practise;
use super::encoding::{Sample, examples};
use super::learning::{Learner, Verdict, shape};
use super::learning_support::{fresh, small, wave};
use super::progress::Outcome;
use super::selection::best;
use crate::domain::{Activities, Features};
use crate::neural::{Adam, Hybrid};

fn silent(count: usize) -> Vec<Sample> {
    (0..count)
        .map(|index| Sample {
            at: index as f64 * 2.0,
            features: Features::default(),
            activities: Activities::default(),
        })
        .collect()
}

fn finish(learner: &mut Learner) -> Verdict {
    loop {
        match learner.step(
            None,
            std::time::Instant::now() + std::time::Duration::from_millis(50),
        ) {
            Verdict::Training => continue,
            other => return other,
        }
    }
}

#[test]
fn lote_vazio_nao_mexe_nos_pesos_nem_no_adam() {
    let plan = small();
    let mut learner = Learner::new(plan);
    wave(&mut learner, 120);
    let rows: Vec<Sample> = learner.samples().copied().collect();
    let lessons = examples(&rows);
    let mut model = fresh(plan);
    let mut adam = Adam::new(plan.rate, model.params.len());
    assert!(practise(&mut model, &mut adam, lessons.iter().take(4), plan.decay) > 0);
    let (before, momentum) = (model.clone(), adam.clone());
    assert_eq!(
        practise(&mut model, &mut adam, std::iter::empty(), plan.decay),
        0
    );
    assert_eq!(model, before);
    assert_eq!(adam, momentum);
}

#[test]
fn rodada_sem_alvos_nao_troca_o_candidato() {
    let plan = small();
    let kept = Hybrid::new(shape(&plan), 9);
    let mut learner = Learner::new(plan);
    learner.resume(silent(plan.minimum + 50), Some(kept.clone()));
    assert!(matches!(
        finish(&mut learner),
        Verdict::Rejected(_, Outcome::NoData)
    ));
    assert_eq!(learner.incumbent(), Some(&kept));
}

#[test]
fn rodada_sem_alvos_e_sem_candidato_nao_cria_um() {
    let plan = small();
    let mut learner = Learner::new(plan);
    learner.resume(silent(plan.minimum + 50), None);
    assert!(matches!(
        finish(&mut learner),
        Verdict::Rejected(_, Outcome::NoData)
    ));
    assert!(learner.incumbent().is_none());
}

#[test]
fn empate_sem_evidencia_mantem_o_anterior() {
    let plan = small();
    let previous = Hybrid::new(shape(&plan), 9);
    let rows = silent(200);
    let (chosen, _) = best(Some(previous.clone()), fresh(plan), &rows, 140);
    assert_eq!(chosen, previous);
}
