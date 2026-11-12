use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::{Message, TrashEvent};
use super::state::Mode;
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::domain::context::Action;
use crate::domain::removal::Removal;

fn shift_delete(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Named(Named::Delete), Modifiers::SHIFT));
}

fn remota(cena: &mut Cena) {
    let lixeira = HomeTrash::new(cena.dir.path().join("Trash"));
    let raiz = Some(cena.dir.path().to_path_buf());
    cena.files.services.disk = Box::new(LocalDisk::new(lixeira).with_remote(raiz));
}

fn aviso(cena: &Cena) -> String {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
        .unwrap_or_default()
}

#[test]
fn shift_delete_exclui_de_vez_so_depois_de_confirmar() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    shift_delete(&mut cena);
    assert_eq!(
        cena.files.mode,
        Mode::ConfirmRemoval(Removal::Delete, vec!["a.txt".to_owned()])
    );
    cena.tecla("n");
    assert!(cena.work().join("a.txt").exists());
    shift_delete(&mut cena);
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("a.txt").exists());
    assert!(!cena.dir.path().join("Trash/files/a.txt").exists());
    assert_eq!(aviso(&cena), "1 item excluído");
}

#[test]
fn excluir_de_vez_apaga_pastas_inteiras_e_nao_entra_no_desfazer() {
    let mut cena = Cena::nova();
    assert!(fs::write(cena.work().join("Fotos/praia.jpg"), "x").is_ok());
    cena.tecla("g");
    shift_delete(&mut cena);
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("Fotos").exists());
    assert!(cena.files.journal.is_empty());
}

#[test]
fn local_sem_lixeira_pede_exclusao_permanente_no_delete() {
    let mut cena = Cena::nova();
    remota(&mut cena);
    cena.tecla("G");
    cena.tecla("d");
    assert_eq!(
        cena.files.mode,
        Mode::ConfirmRemoval(Removal::Delete, vec!["a.txt".to_owned()])
    );
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("a.txt").exists());
    assert!(!cena.dir.path().join("Trash").exists());
}

#[test]
fn menu_de_contexto_oferece_excluir_de_vez() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.mandar(Message::ContextEntry(1));
    cena.mandar(Message::ContextAction(Action::Delete));
    assert_eq!(
        cena.files.mode,
        Mode::ConfirmRemoval(Removal::Delete, vec!["a.txt".to_owned()])
    );
}

#[test]
fn dentro_da_lixeira_shift_delete_nao_age() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("d");
    cena.nomeada(Named::Enter);
    cena.mandar(Message::Trash(TrashEvent::Open));
    cena.tecla("g");
    shift_delete(&mut cena);
    assert_eq!(cena.files.mode, Mode::Browse);
}
