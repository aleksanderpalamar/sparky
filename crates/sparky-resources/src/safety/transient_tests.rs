use super::fake_executor::FakeExecutor;
use super::journal::{Outcome, prepare, restore};
use super::ledger::Ledger;
use crate::domain::{Adjustment, Change, ChangeKind, Identity, IoClass};
use crate::persistence::fake_storage::FakeStorage;
use crate::ports::{ExecError, Executor};

fn idle() -> Adjustment {
    Adjustment {
        target: Identity {
            pid: 900,
            started: 1,
        },
        change: Change::IoPriority(IoClass::Idle),
    }
}

fn io(executor: &FakeExecutor) -> Option<Change> {
    executor
        .values
        .borrow()
        .get(&(900, ChangeKind::IoPriority))
        .cloned()
}

#[test]
fn falha_transitoria_nas_threads_nao_confirma_a_restauracao() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    for error in [ExecError::Denied, ExecError::Failed] {
        *executor.tasks_error.borrow_mut() = Some(error);
        assert_eq!(restore(&executor, &intent), Outcome::Unconfirmed);
    }
}

#[test]
fn processo_encerrado_nao_recebe_escrita() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(
        executor
            .write(900, &Change::IoPriority(IoClass::Idle))
            .is_ok()
    );
    *executor.tasks_error.borrow_mut() = Some(ExecError::Gone);
    assert_eq!(restore(&executor, &intent), Outcome::Gone);
    assert_eq!(io(&executor), Some(Change::IoPriority(IoClass::Idle)));
}

#[test]
fn entrada_retida_em_falha_transitoria_e_recuperada_depois() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(intent).is_ok());
    assert!(
        executor
            .write(900, &Change::IoPriority(IoClass::Idle))
            .is_ok()
    );
    *executor.tasks_error.borrow_mut() = Some(ExecError::Denied);
    assert_eq!(ledger.recover(&executor), vec![Outcome::Unconfirmed]);
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
    *executor.tasks_error.borrow_mut() = None;
    assert_eq!(ledger.recover(&executor), vec![Outcome::Restored]);
    assert_eq!(io(&executor), Some(Change::IoPriority(IoClass::None)));
    assert!(Ledger::open(&storage).entries().is_empty());
}
