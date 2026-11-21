use crate::dock::{Dock, MAX_SIZE, MIN_SIZE};
use crate::parse;

#[test]
fn sem_secao_o_dock_fica_ligado_no_tamanho_do_figma_e_sempre_visivel() {
    let config = parse("").unwrap_or_default();
    assert_eq!(config.dock, Dock::default());
    assert!(config.dock.enabled);
    assert!(!config.dock.autohide);
    assert_eq!(config.dock.size, 64);
}

#[test]
fn le_ligado_tamanho_e_ocultar() {
    let Ok(config) = parse("[dock]\nenabled = false\nsize = 48\nautohide = true\n") else {
        panic!("configuração válida")
    };
    assert!(!config.dock.enabled);
    assert_eq!(config.dock.size, 48);
    assert!(config.dock.autohide);
}

#[test]
fn tamanho_fora_da_faixa_e_trazido_para_dentro() {
    let pequeno = Dock {
        size: 2,
        ..Dock::default()
    };
    let grande = Dock {
        size: 900,
        ..Dock::default()
    };
    assert_eq!(pequeno.slot(), MIN_SIZE);
    assert_eq!(grande.slot(), MAX_SIZE);
}

#[test]
fn campo_desconhecido_no_dock_e_recusado() {
    assert!(parse("[dock]\nmagnify = true\n").is_err());
}
