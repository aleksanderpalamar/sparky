use std::fs;

use super::message::{Message, TrashEvent};
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::domain::toast::ToastKind;

fn na_lixeira(cena: &mut Cena, nomes: &[&str]) {
    let lixeira = HomeTrash::new(cena.dir.path().join("Trash"));
    for nome in nomes {
        let caminho = cena.work().join(nome);
        if !caminho.exists() {
            assert!(fs::write(&caminho, "x").is_ok());
        }
        assert!(lixeira.put(&caminho).is_ok());
    }
    cena.mandar(Message::Trash(TrashEvent::Open));
}

#[test]
fn restaurar_um_item_mostra_o_aviso_com_a_pasta_de_destino() {
    let mut cena = Cena::nova();
    na_lixeira(&mut cena, &["a.txt"]);
    cena.files.listing.select_name("a.txt");
    cena.mandar(Message::Trash(TrashEvent::RestoreSelected));
    let destino = cena.work().join("a.txt");
    assert!(destino.exists());
    let tipo = cena.files.toast.as_ref().map(|toast| toast.kind.clone());
    assert_eq!(tipo, Some(ToastKind::Restored(vec![destino])));
    assert!(cena.files.notice.is_none());
}

#[test]
fn abrir_pasta_pelo_aviso_vai_ate_o_item_restaurado() {
    let mut cena = Cena::nova();
    na_lixeira(&mut cena, &["a.txt"]);
    cena.files.listing.select_name("a.txt");
    cena.mandar(Message::Trash(TrashEvent::RestoreSelected));
    cena.mandar(Message::ToastOpenFolder);
    assert!(!cena.files.in_trash());
    assert_eq!(cena.files.location, cena.work());
    assert_eq!(cena.selecionado().as_deref(), Some("a.txt"));
    assert!(cena.files.toast.is_none());
}

#[test]
fn restaurar_todos_na_mesma_pasta_mostra_o_aviso_com_a_pasta() {
    let mut cena = Cena::nova();
    na_lixeira(&mut cena, &["a.txt", "b.txt"]);
    cena.mandar(Message::Trash(TrashEvent::RestoreAll));
    let pasta = cena
        .files
        .toast
        .as_ref()
        .and_then(|toast| toast.folder().map(|pasta| pasta.to_path_buf()));
    assert_eq!(pasta, Some(cena.work()));
    let quantos = match cena.files.toast.as_ref().map(|toast| &toast.kind) {
        Some(ToastKind::Restored(caminhos)) => caminhos.len(),
        _ => 0,
    };
    assert_eq!(quantos, 2);
}
