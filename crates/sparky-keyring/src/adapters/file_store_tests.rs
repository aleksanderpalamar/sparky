use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::adapters::file_store::{FileStore, location};
use crate::ports::store::VaultStore;

fn modo(caminho: &std::path::Path) -> Option<u32> {
    fs::metadata(caminho).ok().map(|dados| dados.permissions().mode() & 0o777)
}

#[test]
fn o_cofre_fica_na_pasta_de_dados_do_usuario() {
    assert_eq!(location(Some("/dados"), "/home/ana"), Some(PathBuf::from("/dados/sparky/keyrings/login.json")));
    assert_eq!(location(None, "/home/ana"), Some(PathBuf::from("/home/ana/.local/share/sparky/keyrings/login.json")));
    assert_eq!(location(Some("relativo"), "/home/ana"), Some(PathBuf::from("/home/ana/.local/share/sparky/keyrings/login.json")));
    assert_eq!(location(None, ""), None);
}

#[test]
fn sem_arquivo_nao_ha_cofre() {
    let Ok(pasta) = tempfile::tempdir() else { panic!("sem pasta temporária") };
    let cofre = FileStore::new(pasta.path().join("keyrings/login.json"));
    assert_eq!(cofre.load().ok(), Some(None));
}

#[test]
fn grava_so_para_o_dono_e_le_de_volta() {
    let Ok(pasta) = tempfile::tempdir() else { panic!("sem pasta temporária") };
    let cofre = FileStore::new(pasta.path().join("sparky/keyrings/login.json"));
    assert!(cofre.save("primeiro").is_ok());
    assert!(cofre.save("segundo").is_ok());
    assert_eq!(cofre.load().ok().flatten().as_deref(), Some("segundo"));
    assert_eq!(modo(cofre.path()), Some(0o600));
    assert_eq!(cofre.path().parent().and_then(modo), Some(0o700));
    assert!(!cofre.path().with_extension("json.tmp").exists());
}

#[test]
fn uma_pasta_aberta_demais_volta_a_ser_privada() {
    let Ok(pasta) = tempfile::tempdir() else { panic!("sem pasta temporária") };
    let chaves = pasta.path().join("keyrings");
    assert!(fs::create_dir(&chaves).is_ok());
    assert!(fs::set_permissions(&chaves, fs::Permissions::from_mode(0o755)).is_ok());
    let cofre = FileStore::new(chaves.join("login.json"));
    assert!(cofre.save("x").is_ok());
    assert_eq!(modo(&chaves), Some(0o700));
}
