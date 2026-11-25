use super::fake_executor::FakeExecutor;
use super::journal::{Outcome, prepare};
use super::ledger::Ledger;
use crate::domain::{Adjustment, Change, ChangeKind, Identity, IoClass};
use crate::persistence::fake_storage::FakeStorage;
use crate::ports::Executor;

const NAME: &str = "applied.json";

fn idle() -> Adjustment {
    Adjustment {
        target: Identity {
            pid: 900,
            started: 1,
        },
        change: Change::IoPriority(IoClass::Idle),
    }
}

fn bytes(storage: &FakeStorage) -> Option<Vec<u8>> {
    storage.files.borrow().get(NAME).cloned()
}

fn pending(storage: &FakeStorage, executor: &FakeExecutor) {
    let mut ledger = Ledger::open(storage);
    let Ok(intent) = prepare(executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(intent).is_ok());
    assert!(
        executor
            .write(900, &Change::IoPriority(IoClass::Idle))
            .is_ok()
    );
}

#[test]
fn registro_ausente_permite_iniciar() {
    let storage = FakeStorage::default();
    let ledger = Ledger::open(&storage);
    assert!(!ledger.pending());
}

#[test]
fn registro_sem_permissao_fica_intacto_e_bloqueia_ajustes() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    pending(&storage, &executor);
    let before = bytes(&storage);
    *storage.unreadable.borrow_mut() = true;
    let mut ledger = Ledger::open(&storage);
    assert!(ledger.pending());
    assert_eq!(ledger.recover(&executor), vec![Outcome::Unconfirmed]);
    let Ok(other) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(other).is_err());
    assert_eq!(bytes(&storage), before);
    *storage.unreadable.borrow_mut() = false;
    assert_eq!(ledger.recover(&executor), vec![Outcome::Restored]);
    assert!(!ledger.pending());
    assert_eq!(
        executor.values.borrow().get(&(900, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::None))
    );
}

#[test]
fn registro_corrompido_fica_intacto_e_bloqueia_ajustes() {
    let storage = FakeStorage::default();
    storage
        .files
        .borrow_mut()
        .insert(NAME.to_owned(), b"{quebrado".to_vec());
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    assert!(ledger.pending());
    assert_eq!(ledger.recover(&executor), vec![Outcome::Unconfirmed]);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(intent).is_err());
    assert_eq!(bytes(&storage), Some(b"{quebrado".to_vec()));
}
