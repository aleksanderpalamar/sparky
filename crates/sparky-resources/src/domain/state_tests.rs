use super::state::{Authority, Mode, ServiceState, Suspension, Switch, TrainingState};

#[test]
fn so_adaptativo_e_conservador_podem_intervir() {
    assert_eq!(ServiceState::Adaptive.authority(), Authority::Adaptive);
    assert_eq!(
        ServiceState::Conservative.authority(),
        Authority::Conservative
    );
    for state in [
        ServiceState::Disabled,
        ServiceState::Starting,
        ServiceState::Observing,
        ServiceState::Degraded,
        ServiceState::Safe,
    ] {
        assert_eq!(state.authority(), Authority::None, "{state:?}");
    }
}

#[test]
fn o_padrao_e_ligado_em_observacao() {
    assert_eq!(Switch::default(), Switch::On);
    assert_eq!(Mode::default(), Mode::Observation);
}

#[test]
fn o_treinamento_suspenso_leva_o_motivo() {
    let json = serde_json::to_string(&TrainingState::Suspended(Suspension::SensitiveActivity))
        .unwrap_or_default();
    assert_eq!(
        json,
        r#"{"training":"suspended","reason":"sensitive_activity"}"#
    );
    let allowed = serde_json::to_string(&TrainingState::Allowed).unwrap_or_default();
    assert_eq!(allowed, r#"{"training":"allowed"}"#);
}
