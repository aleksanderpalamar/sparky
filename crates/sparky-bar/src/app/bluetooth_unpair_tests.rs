use std::sync::{Arc, Mutex};

use iced::futures::future::BoxFuture;

use super::{
    message::Message,
    panel_state::BluetoothClient,
    tray_test_support::{bar, messages},
    update::update,
};
use crate::domain::{bluetooth::BluetoothRequest, panel::PanelKind};
use crate::ports::bluetooth::BluetoothControl;

const CELULAR: &str = "/org/bluez/hci0/dev_D8_68_A0_A9_E6_0F";

#[derive(Debug, Default)]
struct Control(Mutex<Vec<BluetoothRequest>>);

impl BluetoothControl for Control {
    fn send(&self, request: BluetoothRequest) -> BoxFuture<'_, Result<(), String>> {
        if let Ok(mut sent) = self.0.lock() {
            sent.push(request);
        }
        Box::pin(async { Ok(()) })
    }
}

fn desparear() -> BluetoothRequest {
    BluetoothRequest::Unpair {
        adapter: "/org/bluez/hci0".to_owned(),
        devices: vec![CELULAR.to_owned()],
    }
}

#[test]
fn a_lixeira_pede_confirmacao_e_cancelar_volta_a_linha_normal() {
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Bluetooth)));
    let _ = update(&mut bar, Message::BluetoothUnpair(Some(CELULAR.to_owned())));
    assert_eq!(bar.panels.unpairing.as_deref(), Some(CELULAR));
    let _ = update(&mut bar, Message::BluetoothUnpair(None));
    assert!(bar.panels.unpairing.is_none());
}

#[test]
fn fechar_o_painel_esquece_a_confirmacao_pendente() {
    let mut bar = bar();
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Bluetooth)));
    let _ = update(&mut bar, Message::BluetoothUnpair(Some(CELULAR.to_owned())));
    let _ = messages(update(&mut bar, Message::ClosePanel));
    assert!(bar.panels.unpairing.is_none());
}

#[test]
fn confirmar_envia_a_remocao_e_encerra_a_confirmacao() {
    let mut bar = bar();
    let control = Arc::new(Control::default());
    bar.panels.bluetooth_client = Some(BluetoothClient(control.clone()));
    let _ = messages(update(&mut bar, Message::Panel(PanelKind::Bluetooth)));
    let _ = update(&mut bar, Message::BluetoothUnpair(Some(CELULAR.to_owned())));
    let _ = messages(update(&mut bar, Message::BluetoothRequest(desparear())));
    assert!(bar.panels.unpairing.is_none());
    assert_eq!(*control.0.lock().unwrap(), vec![desparear()]);
}
