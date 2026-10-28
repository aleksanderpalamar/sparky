use std::fs;

use iced::keyboard::key::Named;
use sparky_ui::notice::Notice;

use super::message::Message;
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::domain::context::{Action, Target};

#[test]
fn abre_lixeira_vazia_e_volta_para_a_pasta_pessoal() {
    let mut scene = Cena::nova();
    scene.mandar(Message::OpenTrash);
    assert!(scene.files.in_trash());
    assert!(scene.files.listing.is_empty());
    scene.nomeada(Named::Backspace);
    assert_eq!(scene.files.location, scene.files.home);
}

#[test]
fn restaura_somente_o_item_escolhido_pelo_menu() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let first = scene.work().join("a.txt");
    let second = scene.work().join("b.txt");
    fs::write(&second, "outro").unwrap();
    trash.put(&first).unwrap();
    trash.put(&second).unwrap();
    scene.mandar(Message::OpenTrash);
    scene.files.listing.select_name("a.txt");
    scene.mandar(Message::ContextEntry(scene.files.listing.cursor()));
    assert_eq!(
        scene.files.context.as_ref().map(|menu| menu.target),
        Some(Target::Trashed)
    );
    scene.mandar(Message::ContextAction(Action::Restore));
    assert!(first.exists());
    assert!(!second.exists());
    assert_eq!(scene.files.listing.len(), 1);
}

#[test]
fn restaurar_todos_preserva_conflitos_e_restaura_os_demais() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let first = scene.work().join("a.txt");
    let second = scene.work().join("b.txt");
    fs::write(&second, "antigo").unwrap();
    trash.put(&first).unwrap();
    trash.put(&second).unwrap();
    fs::write(&first, "novo").unwrap();
    scene.mandar(Message::OpenTrash);
    scene.mandar(Message::RestoreAll);
    assert_eq!(fs::read_to_string(&first).unwrap(), "novo");
    assert_eq!(fs::read_to_string(&second).unwrap(), "antigo");
    assert_eq!(scene.files.listing.len(), 1);
    assert!(matches!(scene.files.notice, Some(Notice::Error(_))));
}

#[test]
fn na_lixeira_teclas_de_apagar_e_criar_nao_alteram_itens() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    trash.put(&scene.work().join("a.txt")).unwrap();
    scene.mandar(Message::OpenTrash);
    scene.nomeada(Named::Delete);
    scene.tecla("a");
    assert!(scene.files.in_trash());
    assert!(scene.dir.path().join("Trash/files/a.txt").exists());
    assert!(matches!(scene.files.mode, super::state::Mode::Browse));
}

#[test]
fn restaurar_todos_inclui_itens_ocultos_pela_visualizacao() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let hidden = scene.work().join(".oculto.txt");
    fs::write(&hidden, "segredo").unwrap();
    trash.put(&hidden).unwrap();
    scene.mandar(Message::OpenTrash);
    assert!(scene.files.listing.is_empty());
    assert_eq!(scene.files.listing.total_len(), 1);
    scene.mandar(Message::RestoreAll);
    assert_eq!(fs::read_to_string(hidden).unwrap(), "segredo");
}
