use zbus::fdo::ManagedObjects;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use super::bluez_objects::{ADAPTER, DEVICE, bluetooth};
use crate::domain::bluetooth::Bluetooth;

fn value(value: &str) -> OwnedValue {
    Value::from(value).try_into().unwrap()
}

fn insert(
    objects: &mut ManagedObjects,
    path: &str,
    interface: &str,
    properties: Vec<(&str, OwnedValue)>,
) {
    objects.insert(
        path.try_into().unwrap(),
        [(
            interface.try_into().unwrap(),
            properties
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect(),
        )]
        .into(),
    );
}

#[test]
fn sem_adaptador_nao_exibe_bluetooth() {
    assert_eq!(bluetooth(&ManagedObjects::new()), None);
}

#[test]
fn o_icone_distingue_desligado_ligado_e_conectado() {
    let mut objects = ManagedObjects::new();
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", false.into()), ("Alias", value("Adaptador"))],
    );
    assert_eq!(
        bluetooth(&objects).unwrap().icon(),
        "bluetooth-disabled-symbolic"
    );
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", true.into()), ("Alias", value("Adaptador"))],
    );
    assert_eq!(bluetooth(&objects).unwrap().icon(), "bluetooth-symbolic");
    let path = ObjectPath::try_from("/org/bluez/hci0").unwrap();
    insert(
        &mut objects,
        "/org/bluez/hci0/device",
        DEVICE,
        vec![
            ("Connected", true.into()),
            ("Alias", value("Fone")),
            ("Adapter", Value::from(path).try_into().unwrap()),
        ],
    );
    let state = bluetooth(&objects).unwrap();
    assert_eq!(state.icon(), "bluetooth-active-symbolic");
    assert!(
        state
            .menu()
            .iter()
            .any(|entry| entry.label == "Conectado: Fone")
    );
    assert!(state.menu().iter().all(|entry| !entry.enabled));
}

#[test]
fn adaptadores_e_dispositivos_sao_ordenados_e_conexoes_nao_migram_entre_radios() {
    let mut objects = ManagedObjects::new();
    insert(
        &mut objects,
        "/org/bluez/hci1",
        ADAPTER,
        vec![("Powered", true.into()), ("Alias", value("Segundo"))],
    );
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", false.into()), ("Alias", value("Primeiro"))],
    );
    insert(
        &mut objects,
        "/org/bluez/hci0/device",
        DEVICE,
        vec![
            ("Connected", true.into()),
            ("Alias", value("Conexão antiga")),
            ("Adapter", value("/org/bluez/hci0")),
        ],
    );
    let Bluetooth::Ready(adapters) = bluetooth(&objects).unwrap() else {
        panic!("sem adaptadores")
    };
    assert_eq!(adapters[0].name, "Primeiro");
    assert_eq!(adapters[1].name, "Segundo");
    assert!(adapters.iter().all(|adapter| adapter.connected.is_empty()));
}

#[test]
fn erro_de_tipo_nao_inventa_radio_ligado() {
    let mut objects = ManagedObjects::new();
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", value("sim"))],
    );
    assert_eq!(
        bluetooth(&objects).unwrap().icon(),
        "bluetooth-disabled-symbolic"
    );
    assert!(Bluetooth::Unavailable.title().contains("indisponível"));
}
