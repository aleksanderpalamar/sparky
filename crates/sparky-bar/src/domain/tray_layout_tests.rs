use super::bluetooth::ADDRESS;
use super::indicator::{Role, Status};
use super::tray::{PrimaryAction, TrayIcon, TrayItem};
use super::tray_layout::groups;

fn item(address: &str) -> TrayItem {
    TrayItem {
        address: address.to_owned(),
        title: address.to_owned(),
        icon: TrayIcon::Missing,
        role: Role::Application,
        status: Status::Active,
        primary_action: PrimaryAction::Activate,
        menu_path: None,
        menu: Vec::new(),
    }
}

#[test]
fn rede_bluetooth_e_atencao_permanecem_visiveis_e_aplicativos_ficam_na_gaveta() {
    let mut items = vec![
        item("rede"),
        item(ADDRESS),
        item("steam"),
        item("alerta"),
        item("passivo"),
    ];
    items[0].role = Role::Network;
    items[1].role = Role::Bluetooth;
    items[3].status = Status::Attention;
    items[4].status = Status::Passive;
    let layout = groups(&items);
    assert_eq!(
        layout
            .pinned
            .iter()
            .map(|item| item.address.as_str())
            .collect::<Vec<_>>(),
        vec!["rede", ADDRESS, "alerta"]
    );
    assert_eq!(
        layout
            .drawer
            .iter()
            .map(|item| item.address.as_str())
            .collect::<Vec<_>>(),
        vec!["steam"]
    );
}

#[test]
fn um_applet_bluetooth_ativo_substitui_o_indicador_nativo_sem_duplicar() {
    let mut items = vec![item(ADDRESS), item("blueman")];
    for item in &mut items {
        item.role = Role::Bluetooth;
    }
    assert_eq!(groups(&items).pinned[0].address, "blueman");
    assert_eq!(groups(&items).pinned.len(), 1);
    items[1].status = Status::Passive;
    assert_eq!(groups(&items).pinned[0].address, ADDRESS);
}
