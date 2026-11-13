use iced::keyboard::key::Named;

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::application::Remember;
use crate::domain::context::Action;

fn abrir_janela(cena: &mut Cena) {
    cena.files.listing.select_name("a.txt");
    let indice = cena.files.listing.cursor();
    cena.mandar(Message::ContextEntry(indice));
    cena.mandar(Message::ContextAction(Action::OpenWith));
}

fn selecionado(cena: &Cena) -> Option<String> {
    cena.files
        .open_with
        .as_ref()
        .and_then(|janela| janela.selected.clone())
}

#[test]
fn a_janela_abre_com_o_recomendado_selecionado_e_os_outros_disponiveis() {
    let mut cena = Cena::nova();
    abrir_janela(&mut cena);
    let Some(janela) = cena.files.open_with.as_ref() else {
        panic!("a janela deveria abrir")
    };
    assert_eq!(janela.choices.recommended.len(), 1);
    assert_eq!(janela.choices.others.len(), 1);
    assert_eq!(selecionado(&cena).as_deref(), Some("teste.desktop"));
}

#[test]
fn buscar_filtra_e_seleciona_o_primeiro_e_enter_abre_uma_vez() {
    let mut cena = Cena::nova();
    abrir_janela(&mut cena);
    cena.mandar(Message::OpenWithQuery("outro".to_owned()));
    assert_eq!(selecionado(&cena).as_deref(), Some("outro.desktop"));
    cena.mandar(Message::OpenWithConfirm);
    assert!(cena.files.open_with.is_none());
    assert_eq!(
        cena.escolhas(),
        [("outro.desktop".to_owned(), Remember::Once)]
    );
}

#[test]
fn setas_mudam_a_selecao_e_usar_sempre_vai_junto() {
    let mut cena = Cena::nova();
    abrir_janela(&mut cena);
    cena.nomeada(Named::ArrowDown);
    assert_eq!(selecionado(&cena).as_deref(), Some("outro.desktop"));
    cena.nomeada(Named::ArrowUp);
    assert_eq!(selecionado(&cena).as_deref(), Some("teste.desktop"));
    cena.mandar(Message::OpenWithRemember(true));
    cena.nomeada(Named::Enter);
    assert_eq!(
        cena.escolhas(),
        [("teste.desktop".to_owned(), Remember::Always)]
    );
    assert_eq!(cena.abertos(), vec![cena.work().join("a.txt")]);
}

#[test]
fn clicar_seleciona_e_clique_duplo_abre() {
    let mut cena = Cena::nova();
    abrir_janela(&mut cena);
    cena.mandar(Message::OpenWithSelect("outro.desktop".to_owned()));
    assert_eq!(selecionado(&cena).as_deref(), Some("outro.desktop"));
    assert!(cena.escolhas().is_empty());
    cena.mandar(Message::OpenWithChoice("inexistente.desktop".to_owned()));
    assert!(cena.files.open_with.is_some());
    cena.mandar(Message::OpenWithChoice("outro.desktop".to_owned()));
    assert!(cena.files.open_with.is_none());
}

#[test]
fn busca_sem_resultado_nao_abre_nada() {
    let mut cena = Cena::nova();
    abrir_janela(&mut cena);
    cena.mandar(Message::OpenWithQuery("zzz".to_owned()));
    assert_eq!(selecionado(&cena), None);
    cena.mandar(Message::OpenWithConfirm);
    assert!(cena.files.open_with.is_some());
    assert!(cena.escolhas().is_empty());
}
