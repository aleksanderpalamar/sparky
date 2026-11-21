use sparky_resources::domain::{Activities, Change, IoClass};
use sparky_resources::orchestration::{Decision, Purpose, Verdict};
use sparky_resources::policy::Block;
use sparky_resources::safety::Outcome;

use super::decision::{Lens, Tone, regressions, title, tone};

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
fn cada_veredito_tem_um_tom() {
    assert_eq!(tone(&Verdict::Applied), Tone::Positive);
    assert_eq!(tone(&Verdict::Kept), Tone::Positive);
    assert_eq!(tone(&Verdict::Simulated), Tone::Neutral);
    assert_eq!(tone(&Verdict::Finished(Outcome::Restored)), Tone::Neutral);
    assert_eq!(tone(&Verdict::Blocked(Block::Protected)), Tone::Protective);
    assert_eq!(tone(&Verdict::Reverted(Outcome::Restored)), Tone::Negative);
    assert_eq!(tone(&Verdict::Failed), Tone::Negative);
    assert_eq!(
        tone(&Verdict::Finished(Outcome::Unconfirmed)),
        Tone::Negative
    );
}

#[test]
fn titulo_descreve_o_tipo_de_ajuste() {
    assert_eq!(title(&Change::Nice(10)), "change-nice");
    assert_eq!(title(&Change::Affinity(vec![5, 11])), "change-affinity");
    assert_eq!(title(&Change::IoPriority(IoClass::Idle)), "change-io");
}

#[test]
fn conta_so_os_ajustes_revertidos() {
    let decisions = [
        decision(Verdict::Applied),
        decision(Verdict::Reverted(Outcome::Restored)),
        decision(Verdict::Kept),
        decision(Verdict::Reverted(Outcome::Gone)),
    ];
    assert_eq!(regressions(&decisions), 2);
}

#[test]
fn as_abas_giram_nos_dois_sentidos() {
    assert_eq!(Lens::default(), Lens::Overview);
    assert_eq!(Lens::Overview.next(), Lens::Decisions);
    assert_eq!(Lens::Learning.next(), Lens::Overview);
    assert_eq!(Lens::Overview.previous(), Lens::Learning);
    assert_eq!(
        Lens::ALL.map(Lens::key),
        ["lens-overview", "lens-decisions", "lens-learning"]
    );
}
