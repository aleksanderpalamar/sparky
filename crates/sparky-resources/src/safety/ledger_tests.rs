use super::fake_executor::FakeExecutor;
use super::journal::{Outcome, apply};
use super::ledger::Ledger;
use crate::domain::{Adjustment, Change, ChangeKind, Identity, IoClass};
use crate::persistence::fake_storage::FakeStorage;

fn idle(pid: u32) -> Adjustment {
    Adjustment {
        target: Identity { pid, started: 1 },
        change: Change::IoPriority(IoClass::Idle),
    }
}

#[test]
fn ajustes_sobrevivem_a_queda_do_daemon_e_sao_desfeitos_ao_voltar() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    {
        let mut ledger = Ledger::open(&storage);
        let Ok(applied) = apply(&executor, &idle(900)) else {
            panic!("deveria aplicar")
        };
        assert!(ledger.record(applied).is_ok());
    }
    let mut reopened = Ledger::open(&storage);
    assert_eq!(reopened.entries().len(), 1);
    assert_eq!(reopened.recover(&executor), vec![Outcome::Restored]);
    assert!(reopened.entries().is_empty());
    assert_eq!(
        executor.values.borrow().get(&(900, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::None))
    );
}

#[test]
fn restauracao_nao_confirmada_continua_registrada() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    let Ok(applied) = apply(&executor, &idle(900)) else {
        panic!("deveria aplicar")
    };
    assert!(ledger.record(applied).is_ok());
    *executor.ignore_writes.borrow_mut() = true;
    assert_eq!(ledger.recover(&executor), vec![Outcome::Unconfirmed]);
    assert_eq!(Ledger::open(&storage).entries().len(), 1);
}

#[test]
fn ajuste_encerrado_sai_do_registro() {
    let storage = FakeStorage::default();
    let executor = FakeExecutor::with_tasks(&[900]);
    let mut ledger = Ledger::open(&storage);
    let Ok(applied) = apply(&executor, &idle(900)) else {
        panic!("deveria aplicar")
    };
    assert!(ledger.record(applied.clone()).is_ok());
    assert!(ledger.settle(&applied).is_ok());
    assert!(Ledger::open(&storage).entries().is_empty());
}

#[test]
fn registro_corrompido_comeca_vazio() {
    let storage = FakeStorage::default();
    storage
        .files
        .borrow_mut()
        .insert("applied.json".to_owned(), b"{quebrado".to_vec());
    assert!(Ledger::open(&storage).entries().is_empty());
}
