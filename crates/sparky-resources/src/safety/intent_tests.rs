use super::fake_executor::FakeExecutor;
use super::journal::{Outcome, prepare};
use super::ledger::Ledger;
use crate::domain::{Adjustment, Change, ChangeKind, Identity, IoClass};
use crate::persistence::fake_storage::FakeStorage;
use crate::ports::Executor;

fn adjust(change: Change) -> Adjustment {
    Adjustment {
        target: Identity {
            pid: 900,
            started: 1,
        },
        change,
    }
}

fn idle() -> Adjustment {
    adjust(Change::IoPriority(IoClass::Idle))
}

fn io(executor: &FakeExecutor, task: u32) -> Option<Change> {
    executor
        .values
        .borrow()
        .get(&(task, ChangeKind::IoPriority))
        .cloned()
}

#[test]
fn preparar_le_os_valores_sem_escrever_nada() {
    let executor = FakeExecutor::with_tasks(&[900, 901]);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert_eq!(intent.previous.len(), 2);
    for task in [900, 901] {
        assert_eq!(io(&executor, task), Some(Change::IoPriority(IoClass::None)));
    }
}

#[test]
fn queda_depois_da_primeira_escrita_e_recuperada_no_proximo_inicio() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900, 901]);
    {
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
    }
    let mut reopened = Ledger::open(&storage);
    assert_eq!(reopened.recover(&executor), vec![Outcome::Restored]);
    assert!(reopened.entries().is_empty());
    assert_eq!(io(&executor, 900), Some(Change::IoPriority(IoClass::None)));
}

#[test]
fn queda_antes_de_escrever_tem_recuperacao_idempotente() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    {
        let mut ledger = Ledger::open(&storage);
        let Ok(intent) = prepare(&executor, &idle()) else {
            panic!("deveria preparar")
        };
        assert!(ledger.record(intent).is_ok());
    }
    assert_eq!(
        Ledger::open(&storage).recover(&executor),
        vec![Outcome::Restored]
    );
    assert!(Ledger::open(&storage).recover(&executor).is_empty());
    assert_eq!(io(&executor, 900), Some(Change::IoPriority(IoClass::None)));
}

#[test]
fn concluir_um_ajuste_mantem_os_anteriores_registrados() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    let (Ok(first), Ok(second)) = (
        prepare(&executor, &idle()),
        prepare(&executor, &adjust(Change::Affinity(vec![3]))),
    ) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(first.clone()).is_ok());
    assert!(ledger.record(second.clone()).is_ok());
    assert!(ledger.settle(&second).is_ok());
    assert_eq!(Ledger::open(&storage).entries(), [first]);
}

#[test]
fn falha_ao_gravar_nao_deixa_o_ajuste_no_registro() {
    let storage = FakeStorage::default();
    *storage.refuse.borrow_mut() = true;
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    let Ok(intent) = prepare(&executor, &idle()) else {
        panic!("deveria preparar")
    };
    assert!(ledger.record(intent).is_err());
    assert!(ledger.entries().is_empty());
}
