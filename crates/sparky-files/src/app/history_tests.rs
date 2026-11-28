use std::fs;
use std::path::PathBuf;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::context::{Action, Target};
use crate::domain::history::Visit;
use crate::domain::sidebar_location::SidebarLocation;

fn pastas(cena: &Cena) -> [PathBuf; 4] {
    let [a, b, c, d] = ["A", "B", "C", "D"].map(|nome| cena.work().join(nome));
    for pasta in [&a, &b, &c, &d] {
        assert!(fs::create_dir_all(pasta).is_ok());
    }
    [a, b, c, d]
}

fn alt(cena: &mut Cena, named: Named) {
    cena.mandar(Message::Key(Key::Named(named), Modifiers::ALT));
}

fn ctrl_t(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Character("t".into()), Modifiers::CTRL));
}

#[test]
fn voltar_para_b_e_abrir_d_tira_c_do_avanco() {
    let mut cena = Cena::nova();
    let [a, b, c, d] = pastas(&cena);
    cena.files.go(a.clone());
    cena.files.go(b.clone());
    cena.files.go(c);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, b);
    cena.files.go(d.clone());
    assert_eq!(cena.files.history.next(), None);
    alt(&mut cena, Named::ArrowRight);
    assert_eq!(cena.files.location, d);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, b);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, a);
}

#[test]
fn avancar_volta_para_onde_o_voltar_saiu() {
    let mut cena = Cena::nova();
    let [a, b, ..] = pastas(&cena);
    cena.files.go(a.clone());
    cena.files.go(b.clone());
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, a);
    alt(&mut cena, Named::ArrowRight);
    assert_eq!(cena.files.location, b);
    assert_eq!(cena.files.history.next(), None);
}

#[test]
fn cada_aba_tem_o_proprio_historico() {
    let mut cena = Cena::nova();
    let [a, b, c, _] = pastas(&cena);
    cena.files.go(a.clone());
    ctrl_t(&mut cena);
    assert_eq!(cena.files.history.previous(), None);
    cena.files.go(c.clone());
    cena.mandar(Message::Tab(0));
    cena.files.go(b);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, a);
    cena.mandar(Message::Tab(1));
    assert_eq!(cena.files.location, c);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, a);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, a);
}

#[test]
fn voltar_para_pasta_removida_avisa_e_mantem_o_historico() {
    let mut cena = Cena::nova();
    let [a, b, ..] = pastas(&cena);
    cena.files.go(a.clone());
    cena.files.go(b.clone());
    assert!(fs::remove_dir(&a).is_ok());
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.location, b);
    assert!(cena.files.notice.is_some());
    assert_eq!(cena.files.history.previous(), Some(&Visit::Folder(a)));
}

#[test]
fn recarregar_a_pasta_nao_entra_no_historico() {
    let mut cena = Cena::nova();
    let [a, ..] = pastas(&cena);
    cena.files.go(a.clone());
    cena.files.go(a);
    cena.files.refresh();
    assert_eq!(
        cena.files.history.previous(),
        Some(&Visit::Folder(cena.work()))
    );
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.history.previous(), None);
}

#[test]
fn paginas_da_sidebar_entram_no_historico() {
    let mut cena = Cena::nova();
    cena.files.open_sidebar(SidebarLocation::Favorites);
    alt(&mut cena, Named::ArrowLeft);
    assert_eq!(cena.files.sidebar_page, None);
    assert_eq!(cena.files.location, cena.work());
    alt(&mut cena, Named::ArrowRight);
    assert_eq!(cena.files.sidebar_page, Some(SidebarLocation::Favorites));
}

#[test]
fn menu_da_area_vazia_volta_e_avanca() {
    let mut cena = Cena::nova();
    let [a, ..] = pastas(&cena);
    cena.files.go(a.clone());
    cena.mandar(Message::ContextBackground);
    let rows = cena.files.menu_rows(Target::Background);
    assert!(rows.iter().any(|row| row.offers(Action::Back)));
    assert!(rows.iter().any(|row| row.is_disabled(Action::Forward)));
    cena.mandar(Message::ContextAction(Action::Back));
    assert_eq!(cena.files.location, cena.work());
    cena.mandar(Message::ContextBackground);
    cena.mandar(Message::ContextAction(Action::Forward));
    assert_eq!(cena.files.location, a);
}

#[test]
fn menu_nao_executa_voltar_sem_historico() {
    let mut cena = Cena::nova();
    cena.mandar(Message::ContextBackground);
    cena.mandar(Message::ContextAction(Action::Back));
    assert_eq!(cena.files.location, cena.work());
    assert!(cena.files.notice.is_none());
}
