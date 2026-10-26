use std::collections::HashMap;

use zbus::zvariant::OwnedValue;

use super::relevant;

fn changed(interface: &str, property: &str, invalidated: Vec<&str>) -> zbus::Message {
    let properties: HashMap<&str, OwnedValue> = [(property, true.into())].into();
    zbus::Message::signal(
        "/org/bluez/hci0",
        "org.freedesktop.DBus.Properties",
        "PropertiesChanged",
    )
    .unwrap()
    .build(&(interface, properties, invalidated))
    .unwrap()
}

#[test]
fn mudancas_de_estado_e_propriedades_invalidadas_atualizam_o_indicador() {
    assert!(relevant(&changed(
        "org.bluez.Adapter1",
        "Powered",
        Vec::new()
    )));
    assert!(relevant(&changed(
        "org.bluez.Device1",
        "Connected",
        Vec::new()
    )));
    assert!(relevant(&changed(
        "org.bluez.Device1",
        "RSSI",
        vec!["Alias"]
    )));
}

#[test]
fn anuncios_de_sinal_e_bateria_nao_disparam_novas_consultas() {
    assert!(!relevant(&changed("org.bluez.Device1", "RSSI", Vec::new())));
    assert!(!relevant(&changed(
        "org.bluez.Battery1",
        "Percentage",
        Vec::new()
    )));
}

#[test]
fn remocao_de_objetos_e_reinicio_do_servico_disparam_atualizacao() {
    let removed = zbus::Message::signal(
        "/",
        "org.freedesktop.DBus.ObjectManager",
        "InterfacesRemoved",
    )
    .unwrap()
    .build(&())
    .unwrap();
    let owner = zbus::Message::signal(
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "NameOwnerChanged",
    )
    .unwrap()
    .build(&())
    .unwrap();
    assert!(relevant(&removed));
    assert!(relevant(&owner));
}
