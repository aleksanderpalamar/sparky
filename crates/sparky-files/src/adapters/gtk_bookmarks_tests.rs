use std::fs;
use std::path::PathBuf;

use super::gtk_bookmarks::GtkBookmarks;
use crate::domain::bookmarks::{Bookmark, Target};
use crate::ports::BookmarkStore;

#[test]
fn arquivo_ausente_e_uma_lista_vazia() {
    let dir = tempfile::tempdir().unwrap();
    let store = GtkBookmarks::new(dir.path().join("gtk-3.0/bookmarks"));
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
    let lidos = GtkBookmarks::new(arquivo).load().unwrap();
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
fn gravar_e_ler_de_novo_preserva_a_ordem_e_o_formato_do_gtk() {
    let dir = tempfile::tempdir().unwrap();
    let arquivo = dir.path().join("gtk-3.0/bookmarks");
    let store = GtkBookmarks::new(arquivo.clone());
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
