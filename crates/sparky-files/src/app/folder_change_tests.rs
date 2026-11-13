use std::fs;

use super::message::Message;
use super::test_scene::Cena;

#[test]
fn aviso_de_mudanca_atualiza_itens_novos_e_alterados_no_lugar() {
    let mut scene = Cena::nova();
    fs::write(scene.work().join("novo.txt"), "x").unwrap();
    fs::write(scene.work().join("a.txt"), "conteudo maior").unwrap();
    scene.mandar(Message::FolderChanged);
    let novo = scene.files.listing.find("novo.txt");
    let alterado = scene
        .files
        .listing
        .find("a.txt")
        .and_then(|entry| entry.size());
    assert!(novo.is_some());
    assert_eq!(alterado, Some(14));
}

#[test]
fn aviso_de_mudanca_preserva_o_item_selecionado() {
    let mut scene = Cena::nova();
    scene.files.listing.select_name("a.txt");
    fs::write(scene.work().join("0-primeiro.txt"), "x").unwrap();
    scene.mandar(Message::FolderChanged);
    assert_eq!(scene.selecionado().as_deref(), Some("a.txt"));
}
