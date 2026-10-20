use system_tray::client::{Event, UpdateEvent};

use crate::adapters::tray_events::changes;
use crate::domain::tray::{MenuEntry, PrimaryAction, TrayChange, TrayIcon, TrayItem};

fn updated_item() -> TrayItem {
    TrayItem {
        address: ":1.1".to_owned(),
        title: "Teste".to_owned(),
        icon: TrayIcon::Missing,
        primary_action: PrimaryAction::Activate,
        menu_path: Some("/Menu".to_owned()),
        menu: vec![MenuEntry { id: 1, label: "Novo nome".to_owned(), enabled: false, separator: false }],
    }
}

#[test]
fn propriedades_do_menu_atualizam_o_modelo_pelo_cache_do_cliente() {
    let event = Event::Update(":1.1".to_owned(), UpdateEvent::MenuDiff(Vec::new()));
    let expected = TrayChange::Menu { address: ":1.1".to_owned(), menu: updated_item().menu };
    let found = changes(event, |address| {
        assert_eq!(address, ":1.1");
        Some(updated_item())
    });
    assert_eq!(found, vec![expected]);
}

#[test]
fn propriedades_de_um_item_que_ja_saiu_nao_recriam_o_menu() {
    let event = Event::Update(":1.1".to_owned(), UpdateEvent::MenuDiff(Vec::new()));
    assert!(changes(event, |_| None).is_empty());
}
