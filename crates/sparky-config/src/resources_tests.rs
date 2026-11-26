use crate::parse;
use crate::resources::{ResourceMode, Resources};

#[test]
fn sem_secao_o_gerenciador_fica_ligado_so_observando() {
    let config = parse("").unwrap_or_default();
    assert_eq!(config.resources, Resources::default());
    assert!(config.resources.enabled && config.resources.learning);
    assert_eq!(config.resources.mode, ResourceMode::Observation);
}

#[test]
fn le_modo_aprendizado_e_aplicativos_protegidos() {
    let text = "[resources]\nenabled = false\nmode = \"conservative\"\nlearning = false\nprotected = [\"obs\", \"rustc\"]\n";
    let Ok(config) = parse(text) else {
        panic!("configuração válida")
    };
    assert!(!config.resources.enabled);
    assert_eq!(config.resources.mode, ResourceMode::Conservative);
    assert!(!config.resources.learning);
    assert_eq!(
        config.resources.protected,
        vec!["obs".to_owned(), "rustc".to_owned()]
    );
}

#[test]
fn campo_ou_modo_desconhecido_e_recusado() {
    assert!(parse("[resources]\nturbo = true\n").is_err());
    assert!(parse("[resources]\nmode = \"agressivo\"\n").is_err());
}

#[test]
fn notificacoes_comecam_desligadas_e_podem_ser_ligadas() {
    assert!(!Resources::default().notify);
    let Ok(config) = parse("[resources]\nnotify = true\n") else {
        panic!("configuração válida")
    };
    assert!(config.resources.notify);
}

#[test]
fn aviso_do_obs_comeca_ligado_e_pode_ser_desligado() {
    let config = parse("").unwrap_or_default();
    assert!(config.resources.obs_guide);
    let quiet = parse("[resources]\nobs_guide = false\n").unwrap_or_default();
    assert!(!quiet.resources.obs_guide);
}
