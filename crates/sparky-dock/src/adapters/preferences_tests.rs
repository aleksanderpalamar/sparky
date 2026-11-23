use std::fs;

use sparky_config::Dock;

use super::preferences::read;

#[test]
fn sem_arquivo_vale_o_padrao_e_com_secao_vale_o_arquivo() {
    let Ok(home) = tempfile::tempdir() else {
        panic!("sem diretório temporário");
    };
    let path = home.path().join("sparky.toml");
    assert_eq!(read(&path), Ok(Dock::default()));
    assert!(fs::write(&path, "[dock]\nsize = 48\nautohide = true\n").is_ok());
    assert_eq!(
        read(&path).map(|dock| (dock.size, dock.autohide)),
        Ok((48, true))
    );
}

#[test]
fn arquivo_invalido_vira_erro_sem_derrubar_o_dock() {
    let Ok(home) = tempfile::tempdir() else {
        panic!("sem diretório temporário");
    };
    let path = home.path().join("sparky.toml");
    assert!(fs::write(&path, "[dock]\nmagnify = true\n").is_ok());
    assert!(read(&path).is_err());
}
