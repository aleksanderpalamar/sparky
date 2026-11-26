use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Activities, Authority, Change, ChangeKind, Identity, IoClass};
use crate::persistence::fake_storage::FakeStorage;
use crate::ports::{ExecError, Executor};
use crate::safety::fake_executor::FakeExecutor;
use crate::safety::{Ledger, Outcome, prepare};
use crate::telemetry::consumers::{Consumer, Usage};

fn kept<'a>(
    executor: &'a FakeExecutor,
    storage: &'a FakeStorage,
    start: Instant,
) -> Conductor<&'a FakeExecutor, &'a FakeStorage> {
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
    conductor.tick(&moment(
        start,
        31,
        gaming(),
        10.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(verdicts(&conductor), vec![Verdict::Applied, Verdict::Kept]);
    conductor
}

fn other() -> Vec<(Consumer, Usage)> {
    vec![(
        Consumer {
            identity: Identity {
                pid: 901,
                started: 1,
            },
            parent: 1,
            name: "cargo".to_owned(),
            ticks: 0,
        },
        Usage::cpu(500),
    )]
}

#[test]
fn alvo_encerrado_libera_a_vaga_para_o_proximo_processo() {
    let (storage, executor, start) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
    );
    let mut conductor = kept(&executor, &storage, start);
    executor.gone.borrow_mut().push(900);
    let consumers = other();
    conductor.tick(&moment(
        start,
        40,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert!(!conductor.adjusting());
    assert!(Ledger::open(&storage).entries().is_empty());
    conductor.tick(&moment(
        start,
        41,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(
        verdicts(&conductor),
        vec![
            Verdict::Applied,
            Verdict::Kept,
            Verdict::Finished(Outcome::Gone),
            Verdict::Applied
        ]
    );
}

#[test]
fn falha_transitoria_nao_encerra_o_ajuste_mantido() {
    let (storage, executor, start) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
    );
    let mut conductor = kept(&executor, &storage, start);
    *executor.tasks_error.borrow_mut() = Some(ExecError::Denied);
    let consumers = hog();
    conductor.tick(&moment(
        start,
        40,
        gaming(),
        10.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert!(conductor.adjusting());
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
    assert_eq!(verdicts(&conductor), vec![Verdict::Applied, Verdict::Kept]);
}

#[test]
fn restauracao_pendente_e_tentada_de_novo_ate_confirmar() {
    let (storage, executor, start) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
    );
    {
        let mut ledger = Ledger::open(&storage);
        let target = hog()[0].0.identity;
        let adjustment = crate::domain::Adjustment {
            target,
            change: Change::IoPriority(IoClass::Idle),
        };
        let Ok(intent) = prepare(&executor, &adjustment) else {
            panic!("deveria preparar")
        };
        assert!(ledger.record(intent).is_ok());
        assert!(executor.write(900, &adjustment.change).is_ok());
    }
    *executor.tasks_error.borrow_mut() = Some(ExecError::Failed);
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    assert!(conductor.unconfirmed());
    let calm = Activities::default();
    conductor.tick(&moment(start, 0, calm, 0.0, Authority::None, &[]));
    assert!(conductor.unconfirmed());
    *executor.tasks_error.borrow_mut() = None;
    conductor.tick(&moment(start, 1, calm, 0.0, Authority::None, &[]));
    assert!(!conductor.unconfirmed());
    assert_eq!(
        executor.values.borrow().get(&(900, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::None))
    );
}
