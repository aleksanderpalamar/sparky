use iced::futures::{
    StreamExt,
    executor::block_on,
    stream::{self, BoxStream},
};

use super::bluetooth::{apply, events_from};
use super::tray_actions::open_menu;
use super::tray_test_support::{bar, messages, removes};
use crate::domain::bluetooth::{ADDRESS, Adapter, Bluetooth};
use crate::domain::indicator::Role;
use crate::ports::bluetooth::BluetoothSource;

struct FakeBluetooth;

impl BluetoothSource for FakeBluetooth {
    fn subscribe(&self) -> BoxStream<'static, Option<Bluetooth>> {
        Box::pin(stream::iter([
            Some(Bluetooth::Ready(vec![Adapter {
                name: "Adaptador".to_owned(),
                powered: true,
                connected: vec!["Fone".to_owned()],
            }])),
            None,
        ]))
    }
}

#[test]
fn o_indicador_nativo_acompanha_a_fonte_abstrata_sem_abrir_dbus() {
    let mut bar = bar();
    let events = block_on(events_from(&FakeBluetooth, |_| None).collect::<Vec<_>>());
    assert_eq!(events.len(), 2);
    let mut events = events.into_iter();
    let _ = super::update::update(&mut bar, events.next().unwrap());
    let item = bar.tray().tray().item(ADDRESS).unwrap();
    assert_eq!(item.role, Role::Bluetooth);
    assert_eq!(item.title, "Bluetooth conectado");
    assert!(item.menu.iter().any(|entry| entry.label.contains("Fone")));
    let _ = super::update::update(&mut bar, events.next().unwrap());
    assert!(bar.tray().tray().item(ADDRESS).is_none());
}

#[test]
fn retirar_o_adaptador_fecha_seu_menu_informativo() {
    let mut bar = bar();
    let event = block_on(events_from(&FakeBluetooth, |_| None).next()).unwrap();
    let _ = super::update::update(&mut bar, event);
    let opened = messages(open_menu(&mut bar, ADDRESS.to_owned()));
    assert_eq!(opened.len(), 1);
    let window = bar.tray().menu().unwrap().window;
    let closed = messages(apply(&mut bar, None));
    assert!(removes(&closed, window));
    assert!(bar.tray().menu().is_none());
}
