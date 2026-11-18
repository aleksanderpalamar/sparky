use super::lifecycle::{Budget, Fault, Knowledge, Lifecycle, Link, Signal};
use crate::domain::{Mode, ServiceState, Switch};

fn running(signals: &[Signal]) -> ServiceState {
    let mut lifecycle = Lifecycle::default();
    lifecycle.receive(Signal::Ready);
    for signal in signals {
        lifecycle.receive(*signal);
    }
    lifecycle.state()
}

fn adaptive() -> Vec<Signal> {
    vec![
        Signal::Chose(Mode::Adaptive),
        Signal::Learned(Knowledge::Validated),
    ]
}

#[test]
fn comeca_inicializando_e_depois_observa() {
    let mut lifecycle = Lifecycle::default();
    assert_eq!(lifecycle.state(), ServiceState::Starting);
    lifecycle.receive(Signal::Ready);
    assert_eq!(lifecycle.state(), ServiceState::Observing);
}

#[test]
fn modo_adaptativo_age_com_estatistica_mesmo_sem_modelo_neural() {
    assert_eq!(
        running(&[Signal::Chose(Mode::Adaptive)]),
        ServiceState::Adaptive
    );
    let mut lifecycle = Lifecycle::default();
    lifecycle.receive(Signal::Learned(Knowledge::Validated));
    assert_eq!(lifecycle.knowledge(), Knowledge::Validated);
}

#[test]
fn o_modo_observacao_vale_mesmo_com_conhecimento() {
    let signals = [
        Signal::Chose(Mode::Observation),
        Signal::Learned(Knowledge::Validated),
    ];
    assert_eq!(running(&signals), ServiceState::Observing);
}

#[test]
fn perder_o_contexto_grafico_torna_o_servico_conservador() {
    let mut signals = adaptive();
    signals.push(Signal::Context(Link::Lost));
    assert_eq!(running(&signals), ServiceState::Conservative);
    signals.push(Signal::Context(Link::Available));
    assert_eq!(running(&signals), ServiceState::Adaptive);
}

#[test]
fn estourar_o_orcamento_degrada_ate_voltar_ao_limite() {
    let mut signals = adaptive();
    signals.push(Signal::Spending(Budget::Exceeded));
    assert_eq!(running(&signals), ServiceState::Degraded);
    signals.push(Signal::Spending(Budget::Within));
    assert_eq!(running(&signals), ServiceState::Adaptive);
}

#[test]
fn restauracao_nao_confirmada_prende_no_modo_seguro_ate_liberar() {
    let mut signals = adaptive();
    signals.push(Signal::Failed(Fault::RestoreUnconfirmed));
    signals.push(Signal::Spending(Budget::Within));
    signals.push(Signal::Chose(Mode::Adaptive));
    assert_eq!(running(&signals), ServiceState::Safe);
    signals.push(Signal::Cleared);
    assert_eq!(running(&signals), ServiceState::Adaptive);
}

#[test]
fn desligado_vence_qualquer_outro_estado() {
    let mut signals = adaptive();
    signals.push(Signal::Failed(Fault::RestoreUnconfirmed));
    signals.push(Signal::Turned(Switch::Off));
    assert_eq!(running(&signals), ServiceState::Disabled);
    assert_eq!(Lifecycle::default().switch(), Switch::On);
}
