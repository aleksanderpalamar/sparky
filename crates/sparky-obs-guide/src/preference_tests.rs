use std::fs;

use super::preference::silence;

#[test]
fn nao_avisar_de_novo_grava_a_preferencia_e_mantem_o_resto() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let path = directory.path().join("sparky.toml");
    assert!(
        fs::write(
            &path,
            "[general]\nfrosted_glass = true\n\n[resources]\nnotify = true\n"
        )
        .is_ok()
    );
    assert!(silence(&path).is_ok());
    let Ok(config) = sparky_config::parse(&fs::read_to_string(&path).unwrap_or_default()) else {
        panic!("configuração válida")
    };
    assert!(!config.resources.obs_guide);
    assert!(config.resources.notify);
    assert!(config.general.frosted_glass);
}

#[test]
fn sem_arquivo_de_configuracao_cria_so_a_preferencia() {
    let Ok(directory) = tempfile::tempdir() else {
        return;
    };
    let path = directory.path().join("sparky").join("sparky.toml");
    assert!(silence(&path).is_ok());
    let Ok(config) = sparky_config::parse(&fs::read_to_string(&path).unwrap_or_default()) else {
        panic!("configuração válida")
    };
    assert!(!config.resources.obs_guide);
}
