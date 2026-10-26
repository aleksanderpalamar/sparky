use super::message::Message;
use super::tray_test_support::{connected, item, messages};
use super::update::update;
use crate::domain::indicator::Role;
use crate::domain::tray::TrayChange;

#[test]
fn clique_esquerdo_na_rede_abre_painel_nativo_sem_ativar_applet() {
    let (mut bar, client) = connected(Ok(()));
    let mut network = item(":1.50");
    network.role = Role::Network;
    bar.tray_mut().apply(TrayChange::Added(network));
    let output = messages(update(&mut bar, Message::TrayActivate(":1.50".to_owned())));
    assert!(output.iter().any(|message| matches!(message,
        Message::NewLayerShell { settings, .. } if settings.namespace.as_deref() == Some("sparky-quick-panel")
    )));
    assert!(client.requests().is_empty());
}
