use iced_layershell::reexport::Anchor;
use iced_layershell::settings::StartMode;

use crate::desk::{Desk, settings, window};

#[test]
fn o_app_comeca_sem_camada_e_abre_a_propria_janela() {
    let settings = settings();
    assert!(matches!(
        settings.layer_settings.start_mode,
        StartMode::Background
    ));
    assert_eq!(settings.layer_settings.anchor, Anchor::empty());
}

#[test]
fn a_janela_anuncia_o_app_id_e_o_tamanho() {
    let janela = window("sparky-files", (1000, 640), (480, 320));
    assert_eq!(janela.app_id, Some("sparky-files"));
    assert_eq!(janela.size, Some((1000, 640)));
    assert_eq!(janela.min_size, Some((480, 320)));
    assert!(!janela.client_side_decorations);
}

#[test]
fn fechar_a_janela_principal_encerra_o_app_e_fechar_um_menu_nao() {
    let principal = iced::window::Id::unique();
    let menu = iced::window::Id::unique();
    let desk = Desk::new(principal);
    assert!(desk.is_main(principal));
    assert!(!desk.is_main(menu));
    assert!(desk.ends_with(principal));
    assert!(!desk.ends_with(menu));
}
