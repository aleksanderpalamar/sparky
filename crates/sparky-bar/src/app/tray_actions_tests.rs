use iced_layershell::reexport::KeyboardInteractivity;

use crate::app::message::Message;
use crate::app::tray_actions;
use crate::app::tray_test_support::{bar, item, messages, opened, removes};
use crate::app::update::update;
use crate::domain::tray::TrayChange;

#[test]
fn o_menu_recebe_teclado_para_fechar_com_escape() {
    let mut bar = bar();
    bar.tray_mut().apply(TrayChange::Added(item(":1.1")));
    let output = messages(update(&mut bar, Message::TrayMenu(":1.1".to_owned())));
    assert!(output.iter().any(|message| matches!(message,
        Message::NewLayerShell { settings, .. }
            if settings.keyboard_interactivity == KeyboardInteractivity::Exclusive
                && settings.exclusive_zone.is_none()
    )));
}

#[test]
fn remover_o_item_fecha_tambem_a_superficie_do_menu() {
    let mut bar = bar();
    let window = opened(&mut bar, ":1.1");
    let output = messages(update(&mut bar, Message::Tray(TrayChange::Removed(":1.1".to_owned()))));
    assert!(removes(&output, window));
    assert!(bar.tray().menu().is_none());
}

#[test]
fn remover_outro_item_preserva_o_menu_aberto() {
    let mut bar = bar();
    let window = opened(&mut bar, ":1.1");
    let output = messages(update(&mut bar, Message::Tray(TrayChange::Removed(":1.2".to_owned()))));
    assert!(output.is_empty());
    assert_eq!(bar.tray().menu().map(|menu| menu.window), Some(window));
}

#[test]
fn um_menu_que_fica_vazio_fecha_a_superficie() {
    let mut bar = bar();
    let window = opened(&mut bar, ":1.1");
    let change = TrayChange::Menu { address: ":1.1".to_owned(), menu: Vec::new() };
    let output = messages(update(&mut bar, Message::Tray(change)));
    assert!(removes(&output, window));
}

#[test]
fn uma_entrada_invalida_desabilitada_ou_separador_nao_fecha_o_menu() {
    for entry in [2, 3, 99] {
        let mut bar = bar();
        let window = opened(&mut bar, ":1.1");
        assert!(messages(tray_actions::choose(&mut bar, entry)).is_empty());
        assert_eq!(bar.tray().menu().map(|menu| menu.window), Some(window));
    }
}

#[test]
fn fechar_o_menu_remove_a_superficie_uma_unica_vez() {
    let mut bar = bar();
    let window = opened(&mut bar, ":1.1");
    let output = messages(update(&mut bar, Message::CloseMenu));
    assert!(removes(&output, window));
    assert!(bar.tray().menu().is_none());
    assert!(messages(update(&mut bar, Message::CloseMenu)).is_empty());
}

#[test]
fn abrir_outro_menu_fecha_a_superficie_anterior() {
    let mut bar = bar();
    let previous = opened(&mut bar, ":1.1");
    bar.tray_mut().apply(TrayChange::Added(item(":1.2")));
    let output = messages(update(&mut bar, Message::TrayMenu(":1.2".to_owned())));
    assert!(removes(&output, previous));
    assert_eq!(bar.tray().menu().map(|menu| menu.address.as_str()), Some(":1.2"));
}
