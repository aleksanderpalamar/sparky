use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::brain::Brain;
use super::forgetting::{Forgetting, reply};
use crate::persistence::Knowledge;
use crate::persistence::fake_storage::FakeStorage;
use crate::protocol::{Refusal, Reply};

const LIMIT: Duration = Duration::from_secs(2);

fn answered_by(outcome: bool) -> Reply {
    let forgetting = Arc::new(Forgetting::default());
    let worker = Arc::clone(&forgetting);
    let sampler = thread::spawn(move || {
        while !worker.take() {
            thread::park_timeout(Duration::from_millis(5));
        }
        worker.finish(outcome);
    });
    let answer = reply(&forgetting, LIMIT);
    let _ = sampler.join();
    answer
}

#[test]
fn so_confirma_depois_de_apagar() {
    assert_eq!(answered_by(true), Reply::Done);
}

#[test]
fn falha_ao_apagar_vira_recusa() {
    assert_eq!(
        answered_by(false),
        Reply::Refused {
            reason: Refusal::ForgetFailed
        }
    );
}

#[test]
fn sem_resposta_a_tempo_nao_confirma() {
    let forgetting = Forgetting::default();
    let begun = Instant::now();
    assert_eq!(
        reply(&forgetting, Duration::from_millis(50)),
        Reply::Refused {
            reason: Refusal::ForgetFailed
        }
    );
    assert!(begun.elapsed() < LIMIT);
    assert!(forgetting.take());
    assert!(!forgetting.take());
}

#[test]
fn falha_parcial_ao_apagar_e_informada_e_pode_ser_repetida() {
    let storage = FakeStorage::default();
    {
        let mut knowledge = Knowledge::new(&storage);
        for kind in crate::persistence::Kind::ALL {
            let checkpoint = crate::persistence::Checkpoint {
                kind,
                hardware: 1,
                values: vec![1.0],
            };
            assert!(knowledge.store(&checkpoint).is_ok());
        }
    }
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), 1);
    storage.break_file("history.ckpt");
    assert!(brain.forget().is_err());
    assert_eq!(storage.files.borrow().len(), 1);
    storage.mend();
    assert!(brain.forget().is_ok());
    assert!(storage.files.borrow().is_empty());
}
