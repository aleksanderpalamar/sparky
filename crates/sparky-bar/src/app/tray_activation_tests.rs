use crate::adapters::tray_events::item_of;
use crate::app::message::Message;
use crate::app::tray_test_support::{bar, connected, item, messages, notifier, opened, removes};
use crate::app::update::update;
use crate::domain::tray::{TrayChange, TrayRequest};

#[test]
fn um_item_passivo_fecha_o_menu_ao_sair_da_bandeja_visivel() {
    let mut bar = bar();
    let window = opened(&mut bar, ":1.1");
    let output = messages(update(
        &mut bar,
        Message::Tray(TrayChange::Status {
            address: ":1.1".to_owned(),
            status: crate::domain::indicator::Status::Passive,
        }),
    ));
    assert!(removes(&output, window));
}

#[test]
fn clique_esquerdo_em_item_que_so_tem_menu_abre_o_menu_sem_ativar_o_aplicativo() {
    let mut bar = bar();
    let menu = system_tray::menu::TrayMenu {
        id: 0,
        submenus: vec![system_tray::menu::MenuItem {
            id: 1,
            label: Some("_Abrir".to_owned()),
            enabled: true,
            visible: true,
            ..Default::default()
        }],
    };
    let item = item_of(
        ":1.1".to_owned(),
        notifier(true),
        Some(&menu),
        &crate::adapters::icons::Icons::new(Vec::new(), "hicolor"),
    );
    let _ = bar.tray_mut().apply(TrayChange::Added(item));
    let output = messages(update(&mut bar, Message::TrayActivate(":1.1".to_owned())));
    assert!(
        output
            .iter()
            .any(|message| matches!(message, Message::NewLayerShell { .. }))
    );
    assert_eq!(
        bar.tray().menu().map(|menu| menu.address.as_str()),
        Some(":1.1")
    );
}

#[test]
fn clique_esquerdo_ativa_um_item_comum_uma_unica_vez() {
    let (mut bar, client) = connected(Ok(()));
    let _ = bar.tray_mut().apply(TrayChange::Added(item(":1.1")));
    let output = messages(update(&mut bar, Message::TrayActivate(":1.1".to_owned())));
    assert!(matches!(output.as_slice(), [Message::Ignore]));
    assert_eq!(
        client.requests(),
        vec![TrayRequest::Activate {
            address: ":1.1".to_owned()
        }]
    );
    assert!(bar.tray().menu().is_none());
}

#[test]
fn clique_direito_abre_o_menu_sem_ativar_o_aplicativo() {
    let (mut bar, client) = connected(Ok(()));
    let _ = bar.tray_mut().apply(TrayChange::Added(item(":1.1")));
    let output = messages(update(&mut bar, Message::TrayMenu(":1.1".to_owned())));
    assert!(
        output
            .iter()
            .any(|message| matches!(message, Message::NewLayerShell { .. }))
    );
    assert!(client.requests().is_empty());
}

#[test]
fn ativacao_recusada_abre_o_menu_disponivel() {
    let (mut bar, client) = connected(Err("recusado".to_owned()));
    let _ = bar.tray_mut().apply(TrayChange::Added(item(":1.1")));
    let output = messages(update(&mut bar, Message::TrayActivate(":1.1".to_owned())));
    assert!(matches!(output.as_slice(), [Message::TrayMenu(address)] if address == ":1.1"));
    assert_eq!(client.requests().len(), 1);
    for message in output {
        let _ = messages(update(&mut bar, message));
    }
    assert!(bar.tray().menu().is_some());
}

#[test]
fn escolher_entrada_envia_o_caminho_e_o_id_e_fecha_a_superficie() {
    let (mut bar, client) = connected(Ok(()));
    let window = opened(&mut bar, ":1.1");
    let output = messages(update(&mut bar, Message::TrayChoose(1)));
    assert!(removes(&output, window));
    assert_eq!(
        client.requests(),
        vec![TrayRequest::Select {
            address: ":1.1".to_owned(),
            menu_path: "/Menu".to_owned(),
            entry: 1
        }]
    );
    assert!(bar.tray().menu().is_none());
}

#[test]
fn nao_envia_comandos_para_item_que_ja_saiu() {
    let (mut bar, client) = connected(Ok(()));
    let output = messages(update(&mut bar, Message::TrayActivate(":1.1".to_owned())));
    assert!(output.is_empty());
    assert!(client.requests().is_empty());
}

#[test]
fn entradas_invalidas_nao_enviam_comandos() {
    for entry in [2, 3, 99] {
        let (mut bar, client) = connected(Ok(()));
        opened(&mut bar, ":1.1");
        assert!(messages(update(&mut bar, Message::TrayChoose(entry))).is_empty());
        assert!(client.requests().is_empty());
    }
}
