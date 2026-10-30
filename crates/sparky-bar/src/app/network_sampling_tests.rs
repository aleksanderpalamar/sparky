use std::time::Instant;

use super::{message::Message, state::Bar, tray_test_support::bar, update::update};
use crate::domain::network::{DeviceState, Network};
use crate::domain::panel::PanelKind;
use crate::domain::telemetry::{Ping, Sample};
use crate::ui::panels::fixtures::ready;

fn com_rede_ativa() -> Bar {
    let mut bar = bar();
    bar.panels = ready();
    bar
}

fn amostra() -> Sample {
    Sample {
        interface: "wlan0".to_owned(),
        at: Instant::now(),
        received: 10,
        sent: 20,
        ping: Ping::Reply(4),
    }
}

fn pede_amostra(bar: &mut Bar) -> bool {
    iced_runtime::task::into_stream(update(bar, Message::Tick)).is_some()
}

#[test]
fn amostragem_so_ocorre_com_o_painel_de_rede_aberto_e_uma_por_vez() {
    let mut bar = com_rede_ativa();
    assert!(!pede_amostra(&mut bar));
    let _ = update(&mut bar, Message::Panel(PanelKind::Audio));
    assert!(!pede_amostra(&mut bar));
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    assert!(bar.panels.sampling);
    assert!(!pede_amostra(&mut bar));
    let window = bar.panels.open.unwrap().window;
    let _ = update(&mut bar, Message::NetworkSample(window, Ok(amostra())));
    assert!(!bar.panels.sampling);
    assert_eq!(bar.panels.metrics.as_ref().map(|m| m.received), Some(10));
    assert!(pede_amostra(&mut bar));
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    assert!(bar.panels.open.is_none());
    assert!(!bar.panels.sampling);
    assert!(bar.panels.metrics.is_none());
    assert!(!pede_amostra(&mut bar));
}

#[test]
fn resultado_de_janela_antiga_e_descartado() {
    let mut bar = com_rede_ativa();
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    let antiga = bar.panels.open.unwrap().window;
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    let atual = bar.panels.open.unwrap().window;
    assert_ne!(antiga, atual);
    let _ = update(&mut bar, Message::NetworkSample(antiga, Ok(amostra())));
    assert!(bar.panels.metrics.is_none());
    assert!(bar.panels.sampling);
    let _ = update(
        &mut bar,
        Message::NetworkSample(atual, Err("falhou".to_owned())),
    );
    assert!(!bar.panels.sampling);
    assert!(bar.panels.metrics.is_none());
}

#[test]
fn sem_dispositivo_ativado_nao_mede() {
    let mut bar = com_rede_ativa();
    if let Network::Ready(state) = &mut bar.panels.network {
        state.devices[0].state = DeviceState::Disconnected;
    }
    let _ = update(&mut bar, Message::Panel(PanelKind::Network));
    assert!(bar.panels.open.is_some());
    assert!(!bar.panels.sampling);
    assert!(bar.panels.metrics.is_none());
}
