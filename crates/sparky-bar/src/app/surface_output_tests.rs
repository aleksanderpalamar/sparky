use super::{
    message::Message,
    tray_test_support::{bar, item, messages},
    update::update,
};
use crate::domain::{panel::PanelKind, tray::TrayChange};

fn esquece_a_saida_antes_de_criar(output: &[Message]) -> bool {
    let forget = output
        .iter()
        .position(|message| matches!(message, Message::ForgetLastOutput));
    let create = output
        .iter()
        .position(|message| matches!(message, Message::NewLayerShell { .. }));
    matches!((forget, create), (Some(forget), Some(create)) if forget < create)
}

#[tokio::test]
async fn cada_painel_abre_na_saida_atual_e_nao_na_lembrada() {
    for kind in [
        PanelKind::Clock,
        PanelKind::Audio,
        PanelKind::Network,
        PanelKind::Bluetooth,
    ] {
        let mut bar = bar();
        let output = messages(update(&mut bar, Message::Panel(kind)));
        assert!(esquece_a_saida_antes_de_criar(&output), "{kind:?}");
    }
}

#[test]
fn o_menu_da_bandeja_abre_na_saida_atual_e_nao_na_lembrada() {
    let mut bar = bar();
    bar.tray_mut().apply(TrayChange::Added(item(":1.1")));
    let output = messages(update(&mut bar, Message::TrayMenu(":1.1".to_owned())));
    assert!(esquece_a_saida_antes_de_criar(&output));
}
