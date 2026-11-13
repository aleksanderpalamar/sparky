use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::{Message, TrashEvent};
use super::state::Mode;
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::domain::context::Action;
use crate::domain::toast::ToastKind;

fn shift_delete(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Named(Named::Delete), Modifiers::SHIFT));
}

fn delete(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Named(Named::Delete), Modifiers::empty()));
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

fn excluir_de_vez(nomes: &[&str]) -> Mode {
    Mode::ConfirmDelete(nomes.iter().map(|nome| (*nome).to_owned()).collect())
}

#[test]
fn delete_manda_para_a_lixeira_sem_perguntar_e_mostra_o_aviso() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    delete(&mut cena);
    assert_eq!(cena.files.mode, Mode::Browse);
    assert!(!cena.work().join("a.txt").exists());
    assert!(cena.dir.path().join("Trash/files/a.txt").exists());
    let nomes = match cena.files.toast.as_ref().map(|toast| &toast.kind) {
        Some(ToastKind::Trashed { names, .. }) => names.clone(),
        _ => Vec::new(),
    };
    assert_eq!(nomes, ["a.txt"]);
}

#[test]
fn desfazer_pelo_aviso_restaura_o_item_e_fecha_o_aviso() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    delete(&mut cena);
    cena.mandar(Message::ToastUndo);
    assert_eq!(
        fs::read_to_string(cena.work().join("a.txt"))
            .ok()
            .as_deref(),
        Some("oi")
    );
    assert!(cena.files.toast.is_none());
    assert!(cena.files.journal.is_empty());
}

#[test]
fn desfazer_pelo_aviso_nao_desfaz_uma_acao_mais_nova() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    delete(&mut cena);
    cena.tecla("a");
    cena.mandar(Message::Typed("Nova/".to_owned()));
    cena.mandar(Message::Submit);
    cena.mandar(Message::ToastUndo);
    assert!(cena.work().join("Nova").is_dir());
    assert!(!cena.work().join("a.txt").exists());
    assert!(cena.files.toast.is_none());
}

#[test]
fn o_aviso_fecha_no_x_e_sozinho_depois_de_alguns_segundos() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    delete(&mut cena);
    cena.mandar(Message::ToastDismiss);
    assert!(cena.files.toast.is_none());
    assert!(!cena.work().join("a.txt").exists());
    cena.tecla("g");
    delete(&mut cena);
    for _ in 0..3 {
        cena.mandar(Message::Tick);
    }
    assert!(cena.files.toast.is_none());
}

#[test]
fn shift_delete_abre_a_confirmacao_e_cancelar_mantem_o_item() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    shift_delete(&mut cena);
    assert_eq!(cena.files.mode, excluir_de_vez(&["a.txt"]));
    cena.mandar(Message::DeleteCancel);
    assert_eq!(cena.files.mode, Mode::Browse);
    assert!(cena.work().join("a.txt").exists());
    shift_delete(&mut cena);
    cena.mandar(Message::Escape);
    assert!(cena.work().join("a.txt").exists());
}

#[test]
fn confirmar_exclui_de_vez_sem_passar_pela_lixeira() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    shift_delete(&mut cena);
    cena.mandar(Message::DeleteConfirm);
    assert!(!cena.work().join("a.txt").exists());
    assert!(!cena.dir.path().join("Trash/files/a.txt").exists());
    assert_eq!(aviso(&cena), "1 item excluído");
    assert!(cena.files.toast.is_none());
}

#[test]
fn enter_tambem_confirma_e_apaga_pastas_inteiras_fora_do_desfazer() {
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
    delete(&mut cena);
    assert_eq!(cena.files.mode, excluir_de_vez(&["a.txt"]));
    cena.mandar(Message::DeleteConfirm);
    assert!(!cena.work().join("a.txt").exists());
    assert!(!cena.dir.path().join("Trash").exists());
}

#[test]
fn menu_de_contexto_oferece_excluir_de_vez() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.mandar(Message::ContextEntry(1));
    cena.mandar(Message::ContextAction(Action::Delete));
    assert_eq!(cena.files.mode, excluir_de_vez(&["a.txt"]));
}

#[test]
fn dentro_da_lixeira_shift_delete_nao_age() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    delete(&mut cena);
    cena.mandar(Message::Trash(TrashEvent::Open));
    cena.tecla("g");
    shift_delete(&mut cena);
    assert_eq!(cena.files.mode, Mode::Browse);
}
