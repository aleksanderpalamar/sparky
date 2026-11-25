use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::safety::Ledger;
use crate::safety::fake_executor::FakeExecutor;

#[test]
fn sem_gravar_a_intencao_nenhuma_thread_e_alterada() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    *storage.refuse.borrow_mut() = true;
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(
        affinity(&executor),
        Some(Change::Affinity(vec![0, 1, 2, 3]))
    );
    assert_eq!(verdicts(&conductor), vec![Verdict::Failed]);
}
