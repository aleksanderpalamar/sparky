use super::shadow::{Mode, Scenario, replay};

fn pressure(ticks: usize, high: usize) -> Scenario {
    Scenario {
        stalls: (0..ticks)
            .map(|tick| if tick < high { 40.0 } else { 5.0 })
            .collect(),
        transient: high * 2 <= 10,
        protected_hog: false,
    }
}

#[test]
fn pressao_sustentada_leva_os_dois_modos_a_intervir() {
    let scenario = pressure(40, 40);
    let current = replay(&scenario, Mode::Observed);
    let shadow = replay(&scenario, Mode::Confirmed);
    assert!(current.interventions > 0 && shadow.interventions > 0);
    assert!(shadow.latency >= current.latency);
}

#[test]
fn pressao_passageira_e_contada_como_falso_positivo() {
    let scenario = pressure(40, 2);
    let current = replay(&scenario, Mode::Observed);
    let shadow = replay(&scenario, Mode::Confirmed);
    assert_eq!(current.false_positives, current.interventions);
    assert!(shadow.false_positives <= current.false_positives);
}

#[test]
fn sem_pressao_ninguem_intervem() {
    let scenario = pressure(40, 0);
    for mode in [Mode::Observed, Mode::Confirmed] {
        assert_eq!(replay(&scenario, mode).interventions, 0);
    }
}

#[test]
fn processo_protegido_nunca_e_ajustado() {
    let scenario = Scenario {
        protected_hog: true,
        ..pressure(40, 40)
    };
    for mode in [Mode::Observed, Mode::Confirmed] {
        let result = replay(&scenario, mode);
        assert_eq!(result.interventions, 0);
        assert_eq!(result.protected_writes, 0);
    }
}

#[test]
fn replay_e_deterministico() {
    let scenario = pressure(40, 40);
    assert_eq!(
        replay(&scenario, Mode::Confirmed),
        replay(&scenario, Mode::Confirmed)
    );
}
