use std::fs;
use std::path::PathBuf;

use crate::config::cli::ConfigSource;
use crate::config::load::load;

#[test]
fn sem_arquivo_no_caminho_padrao_valem_os_padroes() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("config.toml");
    assert_eq!(load(&ConfigSource::Default, &caminho), Ok(None));
}

#[test]
fn arquivo_pedido_pela_linha_de_comando_precisa_existir() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let pedido = ConfigSource::Path(pasta.path().join("falta.toml"));
    assert!(load(&pedido, &PathBuf::from("/nao/importa")).is_err());
}

#[test]
fn arquivo_invalido_diz_qual_e_o_arquivo() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("config.toml");
    fs::write(&caminho, "format = \"html\"\n").unwrap_or_else(|error| panic!("{error}"));
    let erro = load(&ConfigSource::Default, &caminho).err();
    assert_eq!(erro.map(|erro| erro.path), Some(caminho));
}

#[test]
fn no_config_ignora_ate_arquivo_invalido() {
    let pasta = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
    let caminho = pasta.path().join("config.toml");
    fs::write(&caminho, "lixo = [").unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(load(&ConfigSource::Skip, &caminho), Ok(None));
}
