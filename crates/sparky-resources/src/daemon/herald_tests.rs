use std::cell::RefCell;

use super::herald::Herald;
use crate::domain::{Activities, Change};
use crate::orchestration::{Decision, Purpose, Verdict};
use crate::ports::Notifier;

#[derive(Default)]
struct Mural {
    titles: RefCell<Vec<String>>,
}

impl Notifier for &Mural {
    fn notify(&self, title: &str, _body: &str) {
        self.titles.borrow_mut().push(title.to_owned());
    }
}

fn decision(verdict: Verdict) -> Decision {
    Decision {
        at_seconds: 0,
        activities: Activities::default(),
        purpose: Purpose::EaseCpu,
        process: "rustc".to_owned(),
        change: Change::Nice(10),
        stall: Some(30.0),
        verdict,
    }
}

#[test]
fn so_avisa_decisoes_novas_e_relevantes() {
    let mural = Mural::default();
    let mut herald = Herald::new(&mural);
    let mut log = vec![decision(Verdict::Simulated), decision(Verdict::Applied)];
    herald.hear(&log, 2, true);
    assert_eq!(mural.titles.borrow().len(), 1);
    herald.hear(&log, 2, true);
    assert_eq!(mural.titles.borrow().len(), 1);
    log.push(decision(Verdict::Kept));
    herald.hear(&log, 3, true);
    assert_eq!(mural.titles.borrow().len(), 1);
}

#[test]
fn desligado_nao_avisa_nem_acumula_para_depois() {
    let mural = Mural::default();
    let mut herald = Herald::new(&mural);
    let log = vec![decision(Verdict::Applied)];
    herald.hear(&log, 1, false);
    herald.hear(&log, 1, true);
    assert!(mural.titles.borrow().is_empty());
}

#[test]
fn registro_cheio_nao_repete_entradas_antigas() {
    let mural = Mural::default();
    let mut herald = Herald::new(&mural);
    let log = vec![decision(Verdict::Applied), decision(Verdict::Applied)];
    herald.hear(&log, 150, true);
    assert_eq!(mural.titles.borrow().len(), 2);
}
