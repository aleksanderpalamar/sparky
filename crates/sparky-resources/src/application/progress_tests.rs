use super::progress::{Outcome, Progress, Round, outcome};
use super::validation::Errors;

fn errors(neural: f64, statistical: f64) -> Errors {
    Errors {
        statistical,
        neural: Some(neural),
        count: 100,
    }
}

#[test]
fn promove_so_quem_vence_a_estatistica_pela_margem() {
    assert_eq!(
        outcome(0.05, &errors(9.0, 10.0), None, true),
        Outcome::Promoted
    );
    assert_eq!(
        outcome(0.05, &errors(9.6, 10.0), None, true),
        Outcome::BehindStatistics
    );
    assert_eq!(
        outcome(0.05, &errors(11.0, 10.0), None, true),
        Outcome::BehindStatistics
    );
}

#[test]
fn precisa_vencer_tambem_o_modelo_ativo() {
    assert_eq!(
        outcome(0.05, &errors(9.0, 10.0), Some(8.5), true),
        Outcome::BehindActive
    );
    assert_eq!(
        outcome(0.05, &errors(8.0, 10.0), Some(8.5), true),
        Outcome::Promoted
    );
}

#[test]
fn modelo_instavel_ou_sem_dados_e_descartado_com_o_motivo_certo() {
    assert_eq!(
        outcome(0.05, &errors(1.0, 10.0), None, false),
        Outcome::Unstable
    );
    assert_eq!(
        outcome(0.05, &errors(f64::NAN, 10.0), None, true),
        Outcome::Unstable
    );
    let empty = Errors {
        count: 0,
        ..errors(1.0, 10.0)
    };
    assert_eq!(outcome(0.05, &empty, None, true), Outcome::NoData);
}

#[test]
fn rodada_guarda_os_erros_e_o_progresso_vai_e_volta_pelo_protocolo() {
    let round = Round::new(90, Outcome::BehindStatistics, &errors(4.1, 3.8));
    assert_eq!(round.neural, Some(4.1));
    assert_eq!(round.statistical, 3.8);
    let progress = Progress {
        collected: 600,
        needed: 600,
        epoch: 3,
        epochs: 20,
        rounds: 2,
        last: Some(round),
    };
    let text = serde_json::to_string(&progress).unwrap_or_default();
    assert!(text.contains(r#""outcome":"behind_statistics""#), "{text}");
    assert_eq!(serde_json::from_str::<Progress>(&text).ok(), Some(progress));
}
