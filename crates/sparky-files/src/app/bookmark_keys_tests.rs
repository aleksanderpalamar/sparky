use std::fs;

use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;
use crate::adapters::bookmark_file::BookmarkFile;
use crate::domain::bookmarks::Bookmark;
use crate::ports::BookmarkStore;

fn gravados(cena: &Cena) -> Vec<String> {
    BookmarkFile::new(cena.dir.path().join("config/sparky/bookmarks"))
        .load()
        .unwrap_or_default()
        .iter()
        .map(Bookmark::name)
        .collect()
}

fn mostrados(cena: &Cena) -> Vec<String> {
    cena.files
        .bookmarks
        .items
        .local()
        .map(|(_, marcador)| marcador.name())
        .collect()
}

fn aviso(cena: &Cena) -> String {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
        .unwrap_or_default()
}

fn ctrl_d(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Character("d".into()), Modifiers::CTRL));
}

#[test]
fn ctrl_d_marca_a_pasta_atual_e_grava_no_arquivo_do_sparky() {
    let mut cena = Cena::nova();
    assert!(cena.files.go(cena.work().join("Fotos")));
    ctrl_d(&mut cena);
    assert_eq!(mostrados(&cena), ["Fotos"]);
    assert_eq!(gravados(&cena), ["Fotos"]);
    assert_eq!(aviso(&cena), "“Fotos” adicionado aos marcadores");
}

#[test]
fn ctrl_d_numa_pasta_ja_marcada_nao_duplica() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("Fotos");
    ctrl_d(&mut cena);
    ctrl_d(&mut cena);
    assert_eq!(gravados(&cena), ["Fotos"]);
    assert_eq!(aviso(&cena), "“Fotos” já está nos marcadores");
}

#[test]
fn ctrl_d_com_uma_pasta_selecionada_marca_a_pasta_e_nao_a_aberta() {
    let mut cena = Cena::nova();
    assert!(fs::create_dir(cena.work().join("Projetos")).is_ok());
    cena.mandar(Message::FolderChanged);
    cena.files.listing.select_name("Projetos");
    ctrl_d(&mut cena);
    assert_eq!(gravados(&cena), ["Projetos"]);
    assert_eq!(aviso(&cena), "“Projetos” adicionado aos marcadores");
}

#[test]
fn ctrl_d_com_um_arquivo_selecionado_marca_a_pasta_aberta() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    ctrl_d(&mut cena);
    assert_eq!(gravados(&cena), ["work"]);
}

#[test]
fn ctrl_d_com_itens_marcados_marca_so_as_pastas_marcadas() {
    let mut cena = Cena::nova();
    assert!(fs::create_dir(cena.work().join("Projetos")).is_ok());
    cena.mandar(Message::FolderChanged);
    for nome in ["Fotos", "Projetos", "a.txt"] {
        cena.files.listing.select_name(nome);
        cena.files.listing.toggle_mark();
    }
    ctrl_d(&mut cena);
    assert_eq!(gravados(&cena), ["Fotos", "Projetos"]);
    assert_eq!(aviso(&cena), "2 pastas adicionadas aos marcadores");
}

#[test]
fn ctrl_d_com_so_arquivos_marcados_avisa_e_nao_marca_nada() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    cena.files.listing.toggle_mark();
    ctrl_d(&mut cena);
    assert!(gravados(&cena).is_empty());
    assert_eq!(aviso(&cena), "Só pastas podem virar marcadores");
}
