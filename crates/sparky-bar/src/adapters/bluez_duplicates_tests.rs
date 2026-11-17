use zbus::fdo::ManagedObjects;

use super::bluez_fixtures::{insert, value};
use super::bluez_objects::{ADAPTER, DEVICE, bluetooth};
use crate::domain::bluetooth::{Bluetooth, Device};

const ANTIGO: &str = "/org/bluez/hci0/dev_7E_4F_15_BF_43_37";
const IDENTIDADE: &str = "/org/bluez/hci0/dev_D8_68_A0_A9_E6_0F";
const OUTRO: &str = "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF";
const NOME: &str = "S22 Ultra de Palamar";

fn celular(objects: &mut ManagedObjects, path: &str, address: &str, connected: bool) {
    insert(
        objects,
        path,
        DEVICE,
        vec![
            ("Address", value(address)),
            ("Alias", value(NOME)),
            ("Icon", value("phone")),
            ("Paired", true.into()),
            ("Connected", connected.into()),
            ("Adapter", value("/org/bluez/hci0")),
        ],
    );
}

fn dispositivos(objects: &ManagedObjects) -> Vec<Device> {
    let Some(Bluetooth::Ready(adapters)) = bluetooth(objects) else {
        panic!("sem adaptador");
    };
    adapters
        .into_iter()
        .flat_map(|adapter| adapter.devices)
        .collect()
}

fn cenario(antigo_conectado: bool) -> ManagedObjects {
    let mut objects = ManagedObjects::new();
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", true.into())],
    );
    celular(&mut objects, ANTIGO, "D8:68:A0:A9:E6:0F", antigo_conectado);
    celular(&mut objects, IDENTIDADE, "D8:68:A0:A9:E6:0F", false);
    objects
}

#[test]
fn o_mesmo_celular_em_dois_objetos_do_bluez_aparece_uma_vez() {
    let lista = dispositivos(&cenario(false));
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].path, IDENTIDADE);
    assert_eq!(lista[0].duplicates, vec![ANTIGO.to_owned()]);
    assert!(lista[0].paired);
}

#[test]
fn o_objeto_conectado_representa_o_aparelho_para_desconectar() {
    let objects = cenario(true);
    let lista = dispositivos(&objects);
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].path, ANTIGO);
    assert!(lista[0].connected);
    let Some(Bluetooth::Ready(adapters)) = bluetooth(&objects) else {
        panic!("sem adaptador");
    };
    assert_eq!(adapters[0].connected, vec![NOME.to_owned()]);
}

#[test]
fn aparelhos_diferentes_com_o_mesmo_nome_continuam_separados() {
    let mut objects = cenario(false);
    celular(&mut objects, OUTRO, "AA:BB:CC:DD:EE:FF", false);
    let caminhos: Vec<_> = dispositivos(&objects).into_iter().map(|d| d.path).collect();
    assert_eq!(caminhos.len(), 2);
    assert!(caminhos.contains(&IDENTIDADE.to_owned()));
    assert!(caminhos.contains(&OUTRO.to_owned()));
}

#[test]
fn objetos_sem_endereco_nao_sao_juntados() {
    let mut objects = ManagedObjects::new();
    insert(
        &mut objects,
        "/org/bluez/hci0",
        ADAPTER,
        vec![("Powered", true.into())],
    );
    for path in [ANTIGO, IDENTIDADE] {
        insert(
            &mut objects,
            path,
            DEVICE,
            vec![
                ("Alias", value(NOME)),
                ("Adapter", value("/org/bluez/hci0")),
            ],
        );
    }
    assert_eq!(dispositivos(&objects).len(), 2);
}
