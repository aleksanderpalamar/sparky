use super::{
    center_geometry,
    message::Message,
    tray_test_support::{bar, messages, removes},
    update::update,
};
use crate::domain::{calendar::Action, panel::PanelKind};
use sparky_notifications::client::Event;

#[tokio::test]
async fn relogio_abre_central_fecha_anterior_e_alterna_sem_mudar_formato() {
    let mut bar = bar();
    let label = bar.clock();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Audio)));
    let previous = bar.panels.open.unwrap().window;
    let output = messages(update(&mut bar, Message::Panel(PanelKind::Clock)));
    assert!(removes(&output, previous));
    assert_eq!(bar.clock(), label);
    assert_eq!(bar.panels.open.unwrap().kind, PanelKind::Clock);
    let window = bar.panels.open.unwrap().window;
    assert!(removes(
        &messages(update(&mut bar, Message::Panel(PanelKind::Clock))),
        window
    ));
}

#[test]
fn calendario_funciona_sem_servico_e_erro_nao_inventa_estado_dnd() {
    let mut bar = bar();
    let before = bar.center.calendar.title();
    let _ = update(&mut bar, Message::Calendar(Action::Next));
    assert_ne!(bar.center.calendar.title(), before);
    let _ = update(
        &mut bar,
        Message::Notifications(Event::Unavailable("offline".to_owned())),
    );
    assert!(bar.center.client.is_none());
    assert!(!bar.center.snapshot.dnd);
    assert_eq!(bar.center.error.as_deref(), Some("offline"));
}

#[tokio::test]
async fn central_cabe_em_monitor_pequeno_e_geometria_atrasada_nao_reabre_janela() {
    assert_eq!(center_geometry::size(640, 480), (620, 422));
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Clock)));
    let window = bar.panels.open.unwrap().window;
    let _ = messages(update(&mut bar, Message::ClosePanel));
    assert!(
        messages(update(
            &mut bar,
            Message::CenterGeometry(window, (620, 422))
        ))
        .is_empty()
    );
}
