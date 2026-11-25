use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use crate::domain::{Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::ports::ExecError;
use crate::safety::Ledger;
use crate::safety::fake_executor::FakeExecutor;

fn applied(executor: &FakeExecutor, storage: &FakeStorage, start: Instant) {
    let consumers = hog();
    let mut conductor = Conductor::new(executor, Ledger::open(storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(affinity(executor), Some(Change::Affinity(vec![3])));
    assert!(conductor.shutdown());
}

#[test]
fn encerramento_desfaz_o_ativo_e_esvazia_o_registro() {
    let (storage, executor) = (FakeStorage::default(), FakeExecutor::with_tasks(&[900]));
    applied(&executor, &storage, Instant::now());
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
    assert!(Ledger::open(&storage).entries().is_empty());
}

#[test]
fn encerramento_com_rollback_pendente_falha_e_conserva_a_entrada() {
    let (storage, executor, start) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
    );
    let consumers = hog();
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    *executor.tasks_error.borrow_mut() = Some(ExecError::Failed);
    assert!(!conductor.shutdown());
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
}
