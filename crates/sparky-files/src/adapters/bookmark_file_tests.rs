use std::fs;
use std::path::{Path, PathBuf};

use super::bookmark_file::{BookmarkFile, location};
use crate::domain::bookmarks::{Bookmark, Target};
use crate::ports::BookmarkStore;

#[test]
fn arquivo_ausente_e_uma_lista_vazia() {
    let dir = tempfile::tempdir().unwrap();
    let store = BookmarkFile::new(dir.path().join("sparky/bookmarks"));
    assert_eq!(store.load(), Ok(Vec::new()));
}

#[test]
fn le_caminhos_com_espaco_rotulos_e_marcadores_remotos() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = dir.path().join("bookmarks");
    fs::write(
        &arquivo,
        "file:///home/p/Meus%20Projetos\nfile:///home/p/obra Obra\nsftp://servidor/srv Servidor\n\n",
    )
    .unwrap();
    let lidos = BookmarkFile::new(arquivo).load().unwrap();
    assert_eq!(
        lidos[0],
        Bookmark::local(PathBuf::from("/home/p/Meus Projetos"))
    );
    assert_eq!(lidos[1].label.as_deref(), Some("Obra"));
    assert_eq!(
        lidos[2].target,
        Target::Remote("sftp://servidor/srv".to_owned())
    );
    assert_eq!(lidos.len(), 3);
}

#[test]
fn gravar_e_ler_de_novo_preserva_a_ordem_e_o_formato() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = dir.path().join("sparky/bookmarks");
    let store = BookmarkFile::new(arquivo.clone());
    let lista = vec![
        Bookmark::local(PathBuf::from("/home/p/Meus Projetos")),
        Bookmark {
            target: Target::Remote("smb://nas/share".to_owned()),
            label: Some("NAS".to_owned()),
        },
    ];
    store.save(&lista).unwrap();
    assert_eq!(
        fs::read_to_string(&arquivo).unwrap(),
        "file:///home/p/Meus%20Projetos\nsmb://nas/share NAS\n"
    );
    assert_eq!(store.load().unwrap(), lista);
}

#[test]
fn o_arquivo_e_proprio_do_sparky_dentro_da_configuracao_do_usuario() {
    assert_eq!(
        location(Path::new("/home/p/.config")),
        PathBuf::from("/home/p/.config/sparky/bookmarks")
    );
}
