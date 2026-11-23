use sparky_ipc::GlassArea;

use super::areas::Areas;

#[test]
fn a_barra_e_os_paineis_sao_vidro_na_superficie_inteira() {
    let areas = Areas::default();
    for namespace in [
        "sparky-bar",
        "sparky-quick-panel",
        "sparky-notifications",
        "sparky-bar-menu",
    ] {
        assert_eq!(areas.of(namespace), Some(GlassArea::Whole));
    }
    assert_eq!(areas.of("sparky-shot"), None);
    assert_eq!(areas.of("waybar"), None);
}

#[test]
fn o_componente_pode_informar_a_area_do_seu_painel() {
    let mut areas = Areas::default();
    let painel = GlassArea::Centered {
        width: 760,
        height: 520,
    };
    assert!(areas.set("sparky-launcher".to_owned(), painel.clone()));
    assert!(!areas.set("sparky-launcher".to_owned(), painel.clone()));
    assert_eq!(areas.of("sparky-launcher"), Some(painel));
}

#[test]
fn so_componentes_do_sparky_podem_pedir_vidro() {
    let mut areas = Areas::default();
    assert!(!areas.set("waybar".to_owned(), GlassArea::Whole));
    assert_eq!(areas.of("waybar"), None);
}
