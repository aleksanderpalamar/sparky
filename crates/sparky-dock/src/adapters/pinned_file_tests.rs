use std::fs;

use super::pinned_file::PinnedFile;
use crate::ports::PinnedStore;

#[test]
fn sem_arquivo_nao_ha_configuracao_salva() {
    let Ok(home) = tempfile::tempdir() else {
        panic!("sem diretório temporário");
    };
    assert_eq!(PinnedFile::new(home.path()).load(), Ok(None));
}

#[test]
fn grava_e_restaura_a_ordem_em_sparky_dock_pinned() {
    let Ok(home) = tempfile::tempdir() else {
        panic!("sem diretório temporário");
    };
    let store = PinnedFile::new(home.path());
    let ids = vec!["sparky-files".to_owned(), "firefox".to_owned()];
    assert_eq!(store.save(&ids), Ok(()));
    assert_eq!(store.load(), Ok(Some(ids)));
    let written = fs::read_to_string(home.path().join("sparky/dock-pinned"));
    assert_eq!(written.ok().as_deref(), Some("sparky-files\nfirefox\n"));
    assert!(!home.path().join("sparky/dock-pinned.new").exists());
}

#[test]
fn arquivo_ilegivel_vira_erro_explicito() {
    let Ok(home) = tempfile::tempdir() else {
        panic!("sem diretório temporário");
    };
    let path = home.path().join("sparky/dock-pinned");
    assert!(fs::create_dir_all(&path).is_ok());
    assert!(PinnedFile::new(home.path()).load().is_err());
}
