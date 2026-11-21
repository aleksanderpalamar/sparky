use std::time::Duration;

use super::evaluation::{Judgement, judge};
use super::log::{Decision, Log, Verdict};
use super::planner::Purpose;
use crate::domain::{Activities, Change};

#[test]
fn espera_o_efeito_antes_de_julgar() {
    assert_eq!(
        judge(40.0, Some(5.0), Duration::from_secs(10)),
        Judgement::Waiting
    );
}

#[test]
fn mantem_so_com_melhora_clara_e_reverte_o_inconclusivo() {
    let after = Duration::from_secs(31);
    assert_eq!(judge(40.0, Some(30.0), after), Judgement::Keep);
    assert_eq!(judge(40.0, Some(35.0), after), Judgement::Revert);
    assert_eq!(judge(40.0, None, after), Judgement::Revert);
}

#[test]
fn o_registro_de_decisoes_e_limitado() {
    let mut log = Log::default();
    for second in 0..150 {
        log.push(Decision {
            at_seconds: second,
            activities: Activities::default(),
            purpose: Purpose::EaseIo,
            process: "rustc".to_owned(),
            change: Change::Nice(10),
            stall: None,
            verdict: Verdict::Simulated,
        });
    }
    let entries = log.entries();
    assert_eq!(entries.len(), 100);
    assert_eq!(entries.first().map(|entry| entry.at_seconds), Some(50));
}
