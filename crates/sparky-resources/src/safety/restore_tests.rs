use super::fake_executor::FakeExecutor;
use super::journal::{Failure, Outcome, apply, restore};
use crate::domain::{Adjustment, Change, ChangeKind, Identity};

fn pinned() -> Adjustment {
    Adjustment {
        target: Identity {
            pid: 900,
            started: 5,
        },
        change: Change::Affinity(vec![3]),
    }
}

fn affinity(executor: &FakeExecutor, task: u32) -> Option<Change> {
    executor
        .values
        .borrow()
        .get(&(task, ChangeKind::Affinity))
        .cloned()
}

fn spawn(executor: &FakeExecutor, task: u32, value: Change) {
    executor.tasks.borrow_mut().push(task);
    executor
        .values
        .borrow_mut()
        .insert((task, ChangeKind::Affinity), value);
}

#[test]
fn threads_criadas_durante_o_ajuste_tambem_sao_restauradas() {
    let executor = FakeExecutor::with_tasks(&[900, 901]);
    let Ok(applied) = apply(&executor, &pinned()) else {
        panic!("deveria aplicar")
    };
    spawn(&executor, 902, Change::Affinity(vec![3]));
    assert_eq!(restore(&executor, &applied), Outcome::Restored);
    for task in [900, 901, 902] {
        assert_eq!(
            affinity(&executor, task),
            Some(Change::Affinity(vec![0, 1, 2, 3])),
            "thread {task}"
        );
    }
}

#[test]
fn thread_nova_com_valor_proprio_nao_e_tocada() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(applied) = apply(&executor, &pinned()) else {
        panic!("deveria aplicar")
    };
    spawn(&executor, 902, Change::Affinity(vec![1]));
    assert_eq!(restore(&executor, &applied), Outcome::Restored);
    assert_eq!(affinity(&executor, 902), Some(Change::Affinity(vec![1])));
}

#[test]
fn thread_nova_que_nao_volta_deixa_a_restauracao_sem_confirmacao() {
    let executor = FakeExecutor::with_tasks(&[900]);
    let Ok(applied) = apply(&executor, &pinned()) else {
        panic!("deveria aplicar")
    };
    spawn(&executor, 902, Change::Affinity(vec![3]));
    *executor.writes_left.borrow_mut() = Some(1);
    assert_eq!(restore(&executor, &applied), Outcome::Unconfirmed);
}

#[test]
fn falha_parcial_sem_desfazer_devolve_o_ajuste_para_recuperacao() {
    let executor = FakeExecutor::with_tasks(&[900, 901]);
    *executor.writes_left.borrow_mut() = Some(1);
    let Err(Failure::Unverified(Outcome::Unconfirmed, applied)) = apply(&executor, &pinned())
    else {
        panic!("deveria ficar sem confirmacao")
    };
    assert_eq!(applied.adjustment, pinned());
    assert_eq!(
        applied.previous,
        vec![
            (900, Change::Affinity(vec![0, 1, 2, 3])),
            (901, Change::Affinity(vec![0, 1, 2, 3])),
        ]
    );
}
