use super::indicator::{Role, icon_names, role};

#[test]
fn rede_e_bluetooth_sao_identificados_sem_depender_do_titulo_traduzido() {
    assert_eq!(role("nm-applet", ""), Role::Network);
    assert_eq!(role("indefinido", "nm-device-wired"), Role::Network);
    assert_eq!(role("blueman", ""), Role::Bluetooth);
    assert_eq!(role("Steam", "steam_tray_mono"), Role::Application);
}

#[test]
fn icones_de_rede_preferem_a_variante_simbolica_preservando_o_estado() {
    for (name, expected) in [
        ("nm-device-wired", "network-wired-symbolic"),
        ("nm-no-connection", "network-offline-symbolic"),
        ("nm-signal-25", "network-wireless-signal-weak-symbolic"),
        (
            "nm-signal-75-secure",
            "network-wireless-signal-good-symbolic",
        ),
        ("bluetooth-disabled", "bluetooth-disabled-symbolic"),
    ] {
        let names = icon_names(name);
        assert_eq!(names.first().map(String::as_str), Some(expected));
        assert_eq!(names.last().map(String::as_str), Some(name));
    }
}

#[test]
fn o_nome_original_permanece_como_fallback_sem_duplicar_o_sufixo() {
    assert_eq!(icon_names("steam"), vec!["steam-symbolic", "steam"]);
    assert_eq!(icon_names("bluetooth-symbolic"), vec!["bluetooth-symbolic"]);
}
