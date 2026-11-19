use sparky_resources::domain::{
    Activity, IoClass, Metric, ServiceState, Suspension, TrainingState,
};

use super::intelligence::{clock, errors, fraction, key, percent, score};

#[test]
fn ausencia_de_dado_nao_vira_zero() {
    assert_eq!(percent(Some(42.25)), Some("42.2%".to_owned()));
    assert_eq!(percent(None), None);
    assert_eq!(percent(Some(f32::NAN)), None);
    assert_eq!(score(Some(3.0)), Some("3.0".to_owned()));
}

#[test]
fn horario_relativo_ao_inicio_do_servico() {
    assert_eq!(clock(3_725), "01:02:05");
}

#[test]
fn estados_viram_chaves_de_traducao() {
    assert_eq!(key("state", &ServiceState::Observing), "state-observing");
    assert_eq!(
        key("activity", &Activity::CriticalPressure),
        "activity-critical-pressure"
    );
    assert_eq!(key("metric", &Metric::CpuBusy), "metric-cpu-busy");
    assert_eq!(
        key("training", &TrainingState::Suspended(Suspension::Budget)),
        "training-suspended"
    );
    assert_eq!(key("io", &IoClass::BestEffort(7)), "io-best-effort");
    assert_eq!(key("io", &IoClass::Idle), "io-idle");
}

#[test]
fn progresso_do_treino_vira_texto_curto() {
    use sparky_resources::application::{Outcome, Round};
    assert_eq!(fraction(312, 600), "312 / 600");
    let round = Round {
        at_seconds: 90,
        outcome: Outcome::BehindStatistics,
        neural: Some(4.104),
        statistical: 3.8,
    };
    assert_eq!(errors(&round), "4.10 / 3.80");
    let empty = Round {
        neural: None,
        ..round
    };
    assert_eq!(errors(&empty), "— / 3.80");
    assert_eq!(key("outcome", &round.outcome), "outcome-behind-statistics");
}
