use super::{
    audio::{AudioDevice, AudioState, Direction},
    network::{NetworkState, Security},
    secret::Secret,
};

#[test]
fn identifica_seguranca_sem_confundir_owe_com_empresa_ou_rede_aberta() {
    assert_eq!(Security::from_flags(0, 0, 0), Security::Open);
    assert_eq!(Security::from_flags(1, 0, 0), Security::Unsupported);
    assert_eq!(Security::from_flags(1, 0x100, 0), Security::Personal);
    assert_eq!(Security::from_flags(1, 0, 0x400), Security::Sae);
    assert_eq!(Security::from_flags(1, 0, 0x2000), Security::Enterprise);
    assert_eq!(Security::from_flags(1, 0, 0x800), Security::Unsupported);
}

#[test]
fn rede_mista_aceita_todos_os_modos_de_autenticacao_anunciados() {
    for (wpa, rsn) in [(0, 0x500), (0x100, 0x400)] {
        assert!(Security::Personal.matches_flags(1, wpa, rsn));
        assert!(Security::Sae.matches_flags(1, wpa, rsn));
        assert!(!Security::Enterprise.matches_flags(1, wpa, rsn));
        assert!(!Security::Open.matches_flags(1, wpa, rsn));
    }
    assert!(Security::Enterprise.matches_flags(1, 0x100, 0x200));
    assert!(Security::Enterprise.matches_flags(1, 0, 0x2000));
}

#[test]
fn modos_de_autenticacao_ausentes_nao_correspondem_ao_perfil() {
    assert!(!Security::Personal.matches_flags(1, 0, 0x400));
    assert!(!Security::Sae.matches_flags(1, 0x100, 0));
    assert!(!Security::Personal.matches_flags(0, 0, 0));
    assert!(!Security::Sae.matches_flags(1, 0, 0x800));
    assert!(!Security::Open.matches_flags(1, 0, 0));
    assert!(Security::Open.matches_flags(0, 0, 0));
}

#[test]
fn valida_senhas_wpa2_e_wpa3_sem_impor_regra_de_psk_ao_sae() {
    assert!(Security::Personal.validate("curta").is_err());
    assert!(Security::Personal.validate("12345678").is_ok());
    assert!(Security::Personal.validate(&"a".repeat(64)).is_ok());
    assert!(Security::Personal.validate(&"z".repeat(64)).is_err());
    assert!(Security::Sae.validate("a").is_ok());
    assert!(Security::Sae.validate("").is_err());
    assert!(Security::Enterprise.validate("qualquer").is_err());
}

#[test]
fn dados_de_entrada_nao_aparecem_no_debug() {
    let secret = Secret::new("senha-confidencial".to_owned());
    assert_eq!(secret.expose(), "senha-confidencial");
    assert_eq!(format!("{secret:?}"), "Secret(<oculto>)");
}

#[test]
fn icone_de_audio_usa_a_saida_padrao_e_respeita_mudo() {
    let mut audio = AudioState {
        default_output: Some("fone".to_owned()),
        ..Default::default()
    };
    assert_eq!(audio.icon(), "audio-volume-muted-symbolic");
    audio.outputs.push(AudioDevice {
        name: "fone".to_owned(),
        label: "Fone".to_owned(),
        channels: 2,
        volume: 50,
        muted: false,
    });
    assert_eq!(audio.selected(Direction::Output).unwrap().name, "fone");
    assert_eq!(audio.icon(), "audio-volume-medium-symbolic");
    audio.outputs[0].muted = true;
    assert_eq!(audio.icon(), "audio-volume-muted-symbolic");
    assert!(audio.selected(Direction::Input).is_none());
    assert_eq!(NetworkState::default().icon(), "network-offline-symbolic");
}
