use super::queue_fixture::{Fixture, events};
use super::rules::{FRAME_INTERVAL, Gate};
use smithay::utils::Transform;
use std::time::Duration;

#[test]
fn captura_antiga_preserva_a_transformacao_do_monitor_aninhado() {
    let fixture = Fixture::new(true);
    fixture
        .output
        .change_current_state(None, Some(Transform::Flipped180), None, None);
    assert_eq!(fixture.job.render_transform(), Transform::Flipped180);
}

#[test]
fn protocolo_novo_mantem_pixels_normais_e_metadado_explicito() {
    let mut fixture = Fixture::new(false);
    fixture
        .output
        .change_current_state(None, Some(Transform::Flipped180), None, None);
    assert_eq!(fixture.job.render_transform(), Transform::Normal);
    assert!(fixture.job.complete(false).is_none());
    assert_eq!(
        events(&mut fixture.display, &mut fixture.socket),
        [0, 1, 2, 3]
    );
}

#[test]
fn transicoes_de_bloqueio_refazem_o_quadro_sem_falha_fatal() {
    for legacy in [false, true] {
        for submitted in [false, true] {
            let mut fixture = Fixture::new(legacy);
            fixture.job.locked = submitted;
            let buffer = fixture.job.buffer.clone();
            let retry = fixture.job.complete(!submitted);
            assert!(retry.is_some(), "legacy={legacy}, submitted={submitted}");
            assert!(events(&mut fixture.display, &mut fixture.socket).is_empty());
            let mut job = retry.unwrap();
            assert_eq!(job.buffer, buffer);
            job.locked = !submitted;
            assert!(job.complete(!submitted).is_none());
            let messages = events(&mut fixture.display, &mut fixture.socket);
            assert_eq!(messages.last(), Some(&if legacy { 2 } else { 3 }));
        }
    }
}

#[test]
fn refazer_quadro_invalida_o_pacing_sem_esperar_mudanca_de_cena() {
    let fixture = Fixture::new(false);
    let (_, data) = fixture.job.session.as_ref().unwrap();
    let data = data.clone();
    data.status
        .lock()
        .unwrap()
        .pace
        .submitted(Duration::ZERO, 7);
    assert_eq!(
        data.status.lock().unwrap().pace.gate(FRAME_INTERVAL, 7),
        Gate::Unchanged
    );
    assert!(fixture.job.complete(true).is_some());
    assert_eq!(
        data.status.lock().unwrap().pace.gate(Duration::ZERO, 7),
        Gate::Ready
    );
}
