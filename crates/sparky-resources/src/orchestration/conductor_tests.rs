use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::policy::Block;
use crate::safety::Ledger;
use crate::safety::fake_executor::FakeExecutor;

#[test]
fn em_observacao_so_simula_e_nao_repete_o_mesmo_aviso() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    for second in 0..3 {
        conductor.tick(&moment(
            start,
            second,
            gaming(),
            40.0,
            Authority::None,
            &consumers,
        ));
    }
    assert_eq!(verdicts(&conductor), vec![Verdict::Simulated]);
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
}

#[test]
fn sem_nenhuma_opcao_reversivel_registra_o_bloqueio() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, Vec::new());
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(
        verdicts(&conductor),
        vec![Verdict::Blocked(Block::NotReversible)]
    );
}
