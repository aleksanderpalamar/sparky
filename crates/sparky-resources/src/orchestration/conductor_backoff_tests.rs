use std::time::Instant;

use super::conductor::Conductor;
use super::conductor_support::*;
use super::log::Verdict;
use crate::domain::{Authority, Identity};
use crate::persistence::fake_storage::FakeStorage;
use crate::safety::fake_executor::FakeExecutor;
use crate::safety::{Ledger, Outcome};
use crate::telemetry::consumers::{Consumer, Usage};

fn reverted_once<'a>(
    conductor: &mut Conductor<&'a FakeExecutor, &'a FakeStorage>,
    start: Instant,
    consumers: &[(Consumer, Usage)],
) {
    for (seconds, stall) in [(0, 40.0), (31, 39.0)] {
        conductor.tick(&moment(
            start,
            seconds,
            gaming(),
            stall,
            Authority::Adaptive,
            consumers,
        ));
    }
}

fn helper() -> (Consumer, Usage) {
    (
        Consumer {
            identity: Identity {
                pid: 901,
                started: 1,
            },
            parent: 1,
            name: "steamwebhelper".to_owned(),
            ticks: 0,
        },
        Usage::cpu(200),
    )
}

#[test]
fn nao_repete_o_ajuste_que_acabou_de_ser_revertido() {
    let (storage, executor, start, consumers) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
        hog(),
    );
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    reverted_once(&mut conductor, start, &consumers);
    for seconds in [95, 200, 400, 590] {
        conductor.tick(&moment(
            start,
            seconds,
            gaming(),
            40.0,
            Authority::Adaptive,
            &consumers,
        ));
    }
    assert_eq!(
        verdicts(&conductor),
        vec![Verdict::Applied, Verdict::Reverted(Outcome::Restored)]
    );
    conductor.tick(&moment(
        start,
        640,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    assert_eq!(verdicts(&conductor).last(), Some(&Verdict::Applied));
}

#[test]
fn com_o_maior_consumidor_em_pausa_tenta_o_seguinte() {
    let (storage, executor, start) = (
        FakeStorage::default(),
        FakeExecutor::with_tasks(&[900]),
        Instant::now(),
    );
    let mut consumers = hog();
    consumers.push(helper());
    let mut conductor = Conductor::new(&executor, Ledger::open(&storage), NO_NICE, vec![3]);
    reverted_once(&mut conductor, start, &consumers);
    conductor.tick(&moment(
        start,
        95,
        gaming(),
        40.0,
        Authority::Adaptive,
        &consumers,
    ));
    let last = conductor.decisions().pop();
    assert_eq!(
        last.map(|decision| (decision.process, decision.verdict)),
        Some(("steamwebhelper".to_owned(), Verdict::Applied))
    );
}
