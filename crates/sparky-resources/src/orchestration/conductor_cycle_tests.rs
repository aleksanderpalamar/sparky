use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Activities, Activity, Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::safety::fake_executor::FakeExecutor;
use crate::safety::{Ledger, Outcome};

#[test]
fn sem_como_desfazer_o_nice_usa_a_afinidade_e_reverte_sem_melhora() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(affinity(&executor), Some(Change::Affinity(vec![3])));
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
    conductor.tick(&moment(
        start,
        31,
        gaming(),
        39.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(
        verdicts(&conductor),
        vec![Verdict::Applied, Verdict::Reverted(Outcome::Restored)]
    );
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
    assert!(Ledger::open(&storage).entries().is_empty());
}

#[test]
fn com_melhora_mantem_e_desfaz_quando_o_jogo_termina() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    conductor.tick(&moment(
        start,
        31,
        gaming(),
        10.0,
        Authority::Adaptive,
        &consumers,
    ));
    conductor.tick(&moment(
        start,
        90,
        Activities::default().with(Activity::Desktop),
        5.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(
        verdicts(&conductor),
        vec![
            Verdict::Applied,
            Verdict::Kept,
            Verdict::Finished(Outcome::Restored)
        ]
    );
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
}

#[test]
fn encerrar_o_servico_desfaz_o_ajuste_ativo() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert!(conductor.release());
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
    assert!(!conductor.unconfirmed());
}

#[test]
fn desligar_ou_voltar_a_observacao_desfaz_o_ajuste_na_hora() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    conductor.tick(&moment(
        start,
        5,
        gaming(),
        40.0,
        Authority::None,
        &consumers,
    ));
    assert_eq!(
        verdicts(&conductor),
        vec![Verdict::Applied, Verdict::Finished(Outcome::Restored)]
    );
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
}
