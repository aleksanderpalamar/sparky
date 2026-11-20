use std::time::Instant;

use super::conductor::{Conductor, Moment};
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Authority, Change};
use crate::persistence::fake_storage::FakeStorage;
use crate::safety::fake_executor::FakeExecutor;
use crate::safety::{Ledger, Outcome};

#[test]
fn falha_parcial_sem_desfazer_entra_em_modo_seguro_e_fica_registrada() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900, 901]),
        Instant::now(),
        hog(),
    );
    *executor.writes_left.borrow_mut() = Some(1);
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    conductor.tick(&moment(
        start,
        0,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(verdicts(&conductor), vec![Verdict::Failed]);
    assert!(conductor.unconfirmed());
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
}

#[test]
fn alvo_que_passa_a_ser_protegido_tem_o_ajuste_desfeito() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    for (seconds, stall) in [(0, 40.0), (31, 10.0)] {
        conductor.tick(&moment(
            start,
            seconds,
            gaming(),
            stall,
            Authority::Adaptive,
            &consumers,
        ));
    }
    conductor.tick(&Moment {
        protected: &[900],
        ..moment(start, 60, gaming(), 10.0, Authority::Adaptive, &consumers)
    });
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
    assert!(Ledger::open(&storage).entries().is_empty());
}
