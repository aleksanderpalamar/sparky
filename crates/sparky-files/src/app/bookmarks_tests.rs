use std::fs;
use std::path::PathBuf;

use iced::Point;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;
use crate::adapters::bookmark_file::BookmarkFile;
use crate::domain::bookmarks::Bookmark;
use crate::domain::drop::DropTarget;
use crate::ports::BookmarkStore;

fn arquivo(cena: &Cena) -> PathBuf {
    cena.dir.path().join("config/sparky/bookmarks")
}

fn gravados(cena: &Cena) -> Vec<String> {
    BookmarkFile::new(arquivo(cena))
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

fn com_marcadores(nomes: &[&str]) -> Cena {
    let mut cena = Cena::nova();
    let lista: Vec<_> = nomes
        .iter()
        .map(|nome| {
            let pasta = cena.work().join(nome);
            assert!(fs::create_dir_all(&pasta).is_ok());
            Bookmark::local(pasta)
        })
        .collect();
    assert!(BookmarkFile::new(arquivo(&cena)).save(&lista).is_ok());
    cena.mandar(Message::Tick);
    cena
}

fn arrastar_item(cena: &mut Cena, nomes: &[&str]) {
    for nome in nomes {
        cena.files.listing.select_name(nome);
        cena.files.listing.toggle_mark();
    }
    cena.mandar(Message::Pointer(Point::new(10.0, 10.0)));
    let indice = cena
        .files
        .listing
        .visible()
        .position(|entry| entry.name == nomes[0])
        .unwrap_or(usize::MAX);
    cena.mandar(Message::Clicked(indice));
    cena.mandar(Message::Pointer(Point::new(40.0, 10.0)));
}

fn arrastar_marcador(cena: &mut Cena, de: usize, para: usize) {
    cena.mandar(Message::Pointer(Point::new(10.0, 300.0)));
    cena.mandar(Message::BookmarkPressed(de));
    cena.mandar(Message::Pointer(Point::new(10.0, 340.0)));
    cena.mandar(Message::Hover(DropTarget::Bookmark(para)));
}

#[test]
fn marcadores_gravados_por_outra_sessao_aparecem_e_nao_se_perdem() {
    let mut cena = com_marcadores(&["Projetos"]);
    assert_eq!(mostrados(&cena), ["Projetos"]);
    let externo = cena.work().join("Musica");
    assert!(fs::create_dir(&externo).is_ok());
    let mut lista = BookmarkFile::new(arquivo(&cena)).load().unwrap_or_default();
    lista.push(Bookmark::local(externo));
    assert!(BookmarkFile::new(arquivo(&cena)).save(&lista).is_ok());
    assert!(cena.files.go(cena.work().join("Fotos")));
    cena.mandar(Message::Key(Key::Character("d".into()), Modifiers::CTRL));
    assert_eq!(gravados(&cena), ["Projetos", "Musica", "Fotos"]);
}

#[test]
fn arrastar_uma_pasta_para_os_marcadores_adiciona_sem_mover() {
    let mut cena = Cena::nova();
    arrastar_item(&mut cena, &["Fotos"]);
    cena.mandar(Message::Hover(DropTarget::Bookmarks));
    cena.concluir(Message::Released);
    assert_eq!(gravados(&cena), ["Fotos"]);
    assert!(cena.work().join("Fotos").is_dir());
    assert!(cena.files.job.is_none());
}

#[test]
fn soltar_sobre_um_marcador_tambem_adiciona_e_ignora_duplicadas_e_arquivos() {
    let mut cena = com_marcadores(&["Projetos"]);
    assert!(fs::create_dir(cena.work().join("Musica")).is_ok());
    cena.mandar(Message::FolderChanged);
    arrastar_item(&mut cena, &["Musica", "Projetos", "a.txt"]);
    cena.mandar(Message::Hover(DropTarget::Bookmark(0)));
    cena.concluir(Message::Released);
    assert_eq!(gravados(&cena), ["Projetos", "Musica"]);
    assert!(cena.work().join("a.txt").is_file());
}

#[test]
fn arrastar_so_arquivos_para_os_marcadores_avisa_e_nao_adiciona() {
    let mut cena = Cena::nova();
    arrastar_item(&mut cena, &["a.txt"]);
    cena.mandar(Message::Hover(DropTarget::Bookmarks));
    cena.concluir(Message::Released);
    assert!(gravados(&cena).is_empty());
    assert_eq!(aviso(&cena), "Só pastas podem virar marcadores");
}

#[test]
fn arrastar_um_marcador_sobre_outro_reordena_e_grava() {
    let mut cena = com_marcadores(&["A", "B", "C"]);
    arrastar_marcador(&mut cena, 0, 2);
    assert!(cena.files.is_bookmark_target(2));
    cena.mandar(Message::Released);
    assert_eq!(mostrados(&cena), ["B", "C", "A"]);
    assert_eq!(gravados(&cena), ["B", "C", "A"]);
}

#[test]
fn esc_ou_soltar_fora_de_um_marcador_nao_reordena() {
    let mut cena = com_marcadores(&["A", "B"]);
    arrastar_marcador(&mut cena, 0, 1);
    cena.mandar(Message::Escape);
    cena.mandar(Message::Released);
    arrastar_marcador(&mut cena, 0, 1);
    cena.mandar(Message::Unhover(DropTarget::Bookmark(1)));
    cena.mandar(Message::Released);
    assert_eq!(gravados(&cena), ["A", "B"]);
    assert_eq!(cena.files.location, cena.work());
}

#[test]
fn clicar_num_marcador_abre_a_pasta() {
    let mut cena = com_marcadores(&["A", "B"]);
    cena.mandar(Message::BookmarkPressed(1));
    cena.mandar(Message::Released);
    assert_eq!(cena.files.location, cena.work().join("B"));
}

#[test]
fn marcador_de_pasta_que_sumiu_avisa_e_fica_onde_esta() {
    let mut cena = com_marcadores(&["A"]);
    assert!(fs::remove_dir(cena.work().join("A")).is_ok());
    cena.mandar(Message::BookmarkPressed(0));
    cena.mandar(Message::Released);
    assert_eq!(cena.files.location, cena.work());
    assert_eq!(aviso(&cena), "A pasta “A” não existe mais");
}
