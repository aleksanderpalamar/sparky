use system_tray::client::{Event, UpdateEvent};

use crate::adapters::tray_events::changes;
use crate::domain::tray::{MenuEntry, PrimaryAction, TrayChange, TrayIcon, TrayItem};

fn updated_item() -> TrayItem {
    TrayItem {
        address: ":1.1".to_owned(),
        title: "Teste".to_owned(),
        icon: TrayIcon::Missing,
        role: crate::domain::indicator::Role::Application,
        status: crate::domain::indicator::Status::Active,
        primary_action: PrimaryAction::Activate,
        menu_path: Some("/Menu".to_owned()),
        menu: vec![MenuEntry {
            id: 1,
            label: "Novo nome".to_owned(),
            enabled: false,
            separator: false,
        }],
    }
}

#[test]
fn propriedades_do_menu_atualizam_o_modelo_pelo_cache_do_cliente() {
    let event = Event::Update(":1.1".to_owned(), UpdateEvent::MenuDiff(Vec::new()));
    let expected = TrayChange::Menu {
        address: ":1.1".to_owned(),
        menu: updated_item().menu,
    };
    let found = changes(
        event,
        |address| {
            assert_eq!(address, ":1.1");
            Some(updated_item())
        },
        &super::icons::Icons::new(Vec::new(), "hicolor"),
    );
    assert_eq!(found, vec![expected]);
}

#[test]
fn propriedades_de_um_item_que_ja_saiu_nao_recriam_o_menu() {
    let event = Event::Update(":1.1".to_owned(), UpdateEvent::MenuDiff(Vec::new()));
    assert!(
        changes(
            event,
            |_| None,
            &super::icons::Icons::new(Vec::new(), "hicolor")
        )
        .is_empty()
    );
}

#[test]
fn atualizacao_de_icone_preserva_o_item_completo_do_cache() {
    let mut item = updated_item();
    item.role = crate::domain::indicator::Role::Network;
    item.icon = TrayIcon::File("/opt/app/private-icon.svg".into());
    let event = Event::Update(
        ":1.1".to_owned(),
        UpdateEvent::Icon {
            icon_name: None,
            icon_pixmap: None,
        },
    );
    let found = changes(
        event,
        |_| Some(item.clone()),
        &super::icons::Icons::new(Vec::new(), "hicolor"),
    );
    assert_eq!(found, vec![TrayChange::Added(item)]);
}

#[test]
fn status_passivo_chega_ao_modelo_mesmo_sem_snapshot() {
    let event = Event::Update(
        ":1.1".to_owned(),
        UpdateEvent::Status(system_tray::item::Status::Passive),
    );
    let found = changes(
        event,
        |_| None,
        &super::icons::Icons::new(Vec::new(), "hicolor"),
    );
    assert_eq!(
        found,
        vec![TrayChange::Status {
            address: ":1.1".to_owned(),
            status: crate::domain::indicator::Status::Passive
        }]
    );
}
