use super::notice::{Notice, notice};
use crate::domain::{Activities, Change};
use crate::orchestration::{Decision, Purpose, Verdict};
use crate::policy::Block;
use crate::safety::Outcome;

fn decision(verdict: Verdict) -> Decision {
    Decision {
        at_seconds: 0,
        activities: Activities::default(),
        purpose: Purpose::EaseCpu,
        process: "steamwebhelper".to_owned(),
        change: Change::Affinity(vec![5, 11]),
        stall: Some(24.0),
        verdict,
    }
}

fn plain(notice: Option<Notice>) -> Option<(String, String)> {
    notice.map(|notice| {
        let clean = |text: String| text.replace(['\u{2068}', '\u{2069}'], "");
        (clean(notice.title), clean(notice.body))
    })
}

#[test]
fn ajuste_aplicado_explica_o_que_foi_feito() {
    assert_eq!(
        plain(notice(&decision(Verdict::Applied))),
        Some((
            "Otimização aplicada".to_owned(),
            "Afinidade de CPU ajustada em steamwebhelper (CPU 5, 11) para aliviar a CPU."
                .to_owned()
        ))
    );
}

#[test]
fn reversao_e_restauracao_nao_confirmada_tambem_avisam() {
    let reverted = plain(notice(&decision(Verdict::Reverted(Outcome::Restored))));
    assert_eq!(
        reverted.map(|(title, _)| title),
        Some("Ajuste revertido".to_owned())
    );
    let unconfirmed = plain(notice(&decision(Verdict::Finished(Outcome::Unconfirmed))));
    assert_eq!(
        unconfirmed.map(|(title, _)| title),
        Some("Restauração não confirmada".to_owned())
    );
}

#[test]
fn decisoes_rotineiras_nao_viram_notificacao() {
    for verdict in [
        Verdict::Simulated,
        Verdict::Kept,
        Verdict::Failed,
        Verdict::Blocked(Block::Cooldown),
        Verdict::Finished(Outcome::Restored),
        Verdict::Reverted(Outcome::Unconfirmed),
    ] {
        let expected = matches!(verdict, Verdict::Reverted(_));
        assert_eq!(
            notice(&decision(verdict.clone())).is_some(),
            expected,
            "{verdict:?}"
        );
    }
}
