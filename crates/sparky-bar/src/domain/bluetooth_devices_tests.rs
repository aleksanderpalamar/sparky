use super::bluetooth::{BluetoothRequest, Device, DeviceKind};
use super::bluetooth_devices::{Record, physical};

fn registro(path: &str, address: Option<&str>, paired: bool, connected: bool) -> Record {
    Record {
        device: Device {
            path: path.to_owned(),
            name: "Fone".to_owned(),
            paired,
            connected,
            kind: DeviceKind::Audio,
            duplicates: Vec::new(),
        },
        address: address.map(str::to_owned),
    }
}

#[test]
fn o_objeto_com_o_caminho_do_endereco_vence_quando_nada_esta_conectado() {
    let lista = physical(vec![
        registro(
            "/org/bluez/hci0/dev_7E_00",
            Some("D8:68:A0:A9:E6:0F"),
            true,
            false,
        ),
        registro(
            "/org/bluez/hci0/dev_D8_68_A0_A9_E6_0F",
            Some("d8:68:a0:a9:e6:0f"),
            false,
            false,
        ),
    ]);
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].path, "/org/bluez/hci0/dev_D8_68_A0_A9_E6_0F");
    assert!(lista[0].paired);
}

#[test]
fn o_objeto_conectado_vence_e_o_estado_e_somado() {
    let lista = physical(vec![
        registro("/org/bluez/hci0/dev_D8_68", Some("D8:68"), true, false),
        registro("/org/bluez/hci0/dev_7E_4F", Some("D8:68"), false, true),
    ]);
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].path, "/org/bluez/hci0/dev_7E_4F");
    assert!(lista[0].paired && lista[0].connected);
}

#[test]
fn enderecos_diferentes_ou_ausentes_nao_se_juntam() {
    let lista = physical(vec![
        registro("/a", Some("AA:AA"), true, false),
        registro("/b", Some("BB:BB"), true, false),
        registro("/c", None, true, false),
        registro("/d", None, true, false),
    ]);
    assert_eq!(lista.len(), 4);
}

#[test]
fn a_escolha_nao_depende_da_ordem_dos_objetos() {
    let primeiro = registro("/org/bluez/hci0/dev_11", Some("D8:68"), true, false);
    let segundo = registro("/org/bluez/hci0/dev_22", Some("D8:68"), true, false);
    let ida = physical(vec![primeiro.clone(), segundo.clone()]);
    let volta = physical(vec![segundo, primeiro]);
    assert_eq!(ida, volta);
}

#[test]
fn desparear_remove_o_objeto_escolhido_e_os_registros_duplicados() {
    let lista = physical(vec![
        registro("/org/bluez/hci0/dev_7E_00", Some("D8:68"), true, false),
        registro("/org/bluez/hci0/dev_D8_68", Some("D8:68"), true, false),
        registro("/org/bluez/hci0/dev_11", Some("11:11"), true, false),
    ]);
    assert_eq!(lista[0].duplicates, vec!["/org/bluez/hci0/dev_7E_00"]);
    assert!(lista[1].duplicates.is_empty());
    assert_eq!(
        lista[0].unpair("/org/bluez/hci0"),
        BluetoothRequest::Unpair {
            adapter: "/org/bluez/hci0".to_owned(),
            devices: vec![
                "/org/bluez/hci0/dev_D8_68".to_owned(),
                "/org/bluez/hci0/dev_7E_00".to_owned(),
            ],
        }
    );
}
