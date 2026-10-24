use super::frame_state::{FrameError, FrameState};
use super::rules::{Gate, Pace, admits, valid_damage};
use std::time::Duration;

#[test]
fn captura_espera_mudancas_e_respeita_o_limite_de_sessenta_quadros() {
    let mut pace = Pace::default();
    assert_eq!(pace.gate(Duration::ZERO, 1), Gate::Ready);
    pace.submitted(Duration::ZERO, 1);
    assert_eq!(pace.gate(Duration::from_secs(1), 1), Gate::Unchanged);
    assert!(matches!(
        pace.gate(Duration::from_millis(1), 2),
        Gate::Later(_)
    ));
    assert_eq!(pace.gate(Duration::from_millis(17), 2), Gate::Ready);
}

#[test]
fn produtor_lento_nao_aumenta_a_fila_sem_limite() {
    assert!(admits(3, 4));
    assert!(!admits(4, 4));
    assert!(!admits(usize::MAX, 1));
}

#[test]
fn danos_validos_seguem_o_protocolo_sem_exigir_coordenadas_do_monitor() {
    assert!(valid_damage(0, 0, 1920, 1080));
    assert!(valid_damage(5000, 10, 30, 40));
    assert!(!valid_damage(-1, 0, 10, 10));
    assert!(!valid_damage(0, 0, 0, 10));
    assert!(!valid_damage(0, 0, 10, -1));
}

#[test]
fn um_quadro_so_pode_ser_capturado_uma_vez_com_buffer_anexado() {
    let mut frame = FrameState::default();
    assert_eq!(frame.capture(), Err(FrameError::NoBuffer));
    assert_eq!(frame.attach("primeiro"), Ok(()));
    assert_eq!(frame.attach("segundo"), Ok(()));
    assert_eq!(frame.capture(), Ok("segundo"));
    assert_eq!(frame.capture(), Err(FrameError::AlreadyCaptured));
    assert_eq!(frame.attach("tardio"), Err(FrameError::AlreadyCaptured));
    assert_eq!(frame.damage(0, 0, 10, 10), Err(FrameError::AlreadyCaptured));
}

#[test]
fn dano_invalido_e_detectado_antes_de_qualquer_acesso_ao_renderizador() {
    let frame = FrameState::<()>::default();
    assert_eq!(frame.damage(-1, 0, 5, 5), Err(FrameError::InvalidDamage));
    assert_eq!(frame.damage(0, 0, 5, 5), Ok(()));
}

#[test]
fn quadro_de_antes_do_bloqueio_nao_e_publicado_depois() {
    assert!(!super::rules::can_publish(false, true));
    assert!(!super::rules::can_publish(true, false));
    assert!(super::rules::can_publish(true, true));
    assert!(super::rules::can_publish(false, false));
}

#[test]
fn captura_mais_urgente_antecipa_o_timer_sem_atrasar_outra_sessao() {
    use super::rules::replaces_wake;
    assert!(replaces_wake(None, Duration::from_millis(16)));
    assert!(replaces_wake(
        Some(Duration::from_millis(16)),
        Duration::from_millis(2)
    ));
    assert!(!replaces_wake(
        Some(Duration::from_millis(2)),
        Duration::from_millis(16)
    ));
}
