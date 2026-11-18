use super::fake_executor::FakeExecutor;
use super::journal::{Failure, Outcome, apply, restore};
use crate::domain::{Adjustment, Change, ChangeKind, Identity, IoClass};
use crate::ports::ExecError;

fn idle_io() -> Adjustment {
    Adjustment {
        target: Identity {
            pid: 900,
            started: 5,
        },
        change: Change::IoPriority(IoClass::Idle),
    }
}

#[test]
fn aplica_em_todas_as_threads_e_restaura_o_estado_anterior() {
    let executor = FakeExecutor::with_tasks(&[900, 901]);
    let Ok(applied) = apply(&executor, &idle_io()) else {
        panic!("deveria aplicar")
    };
    assert_eq!(applied.previous.len(), 2);
    assert_eq!(
        executor.values.borrow().get(&(901, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::Idle))
    );
    assert_eq!(restore(&executor, &applied), Outcome::Restored);
    assert_eq!(
        executor.values.borrow().get(&(901, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::None))
    );
}

#[test]
fn processo_que_terminou_nao_precisa_de_restauracao() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(applied) = apply(&executor, &idle_io()) else {
        panic!("deveria aplicar")
    };
    executor.tasks.borrow_mut().clear();
    assert_eq!(restore(&executor, &applied), Outcome::Gone);
}

#[test]
fn mudanca_feita_por_outro_componente_e_preservada() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(applied) = apply(&executor, &idle_io()) else {
        panic!("deveria aplicar")
    };
    executor.values.borrow_mut().insert(
        (900, ChangeKind::IoPriority),
        Change::IoPriority(IoClass::BestEffort(2)),
    );
    assert_eq!(restore(&executor, &applied), Outcome::ChangedElsewhere);
    assert_eq!(
        executor.values.borrow().get(&(900, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::BestEffort(2)))
    );
}

#[test]
fn escrita_negada_nao_deixa_ajuste_pela_metade() {
    let executor = FakeExecutor::with_tasks(&[900]);
    *executor.deny.borrow_mut() = true;
    assert_eq!(
        apply(&executor, &idle_io()),
        Err(Failure::Refused(ExecError::Denied))
    );
    assert_eq!(
        executor.values.borrow().get(&(900, ChangeKind::IoPriority)),
        Some(&Change::IoPriority(IoClass::None))
    );
}

#[test]
fn ajuste_que_nao_pegou_e_desfeito_e_recusado() {
    let executor = FakeExecutor::with_tasks(&[900]);
    *executor.ignore_writes.borrow_mut() = true;
    assert_eq!(
        apply(&executor, &idle_io()),
        Err(Failure::Refused(ExecError::Failed))
    );
}

#[test]
fn restauracao_sem_confirmacao_e_informada() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(applied) = apply(&executor, &idle_io()) else {
        panic!("deveria aplicar")
    };
    *executor.ignore_writes.borrow_mut() = true;
    assert_eq!(restore(&executor, &applied), Outcome::Unconfirmed);
}
