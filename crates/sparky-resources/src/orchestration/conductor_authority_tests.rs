use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::safety::fake_executor::FakeExecutor;
use crate::safety::{Ledger, Outcome};

#[test]
fn afinidade_mantida_e_desfeita_ao_passar_para_conservador() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    for (seconds, stall, authority) in [
        (0, 40.0, Authority::Adaptive),
        (31, 10.0, Authority::Adaptive),
        (40, 10.0, Authority::Conservative),
    ] {
        conductor.tick(&moment(
            start,
            seconds,
            gaming(),
            stall,
            authority,
            &consumers,
        ));
    }
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
fn sem_ajustes_novos_enquanto_ha_rollback_pendente() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    storage
        .files
        .borrow_mut()
        .insert("applied.json".to_owned(), b"[]".to_vec());
    *storage.unreadable.borrow_mut() = true;
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert!(verdicts(&conductor).is_empty());
    assert!(conductor.unconfirmed());
    *storage.unreadable.borrow_mut() = false;
    conductor.tick(&moment(
        start,
        1,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert!(!conductor.unconfirmed());
    assert_eq!(verdicts(&conductor), vec![Verdict::Applied]);
}
