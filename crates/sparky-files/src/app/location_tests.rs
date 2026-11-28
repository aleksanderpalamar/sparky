use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use crate::domain::context::{Action, Target};

fn ctrl_l(cena: &mut Cena) {
    cena.mandar(Message::Key(Key::Character("l".into()), Modifiers::CTRL));
}

fn tab(cena: &mut Cena, modifiers: Modifiers) {
    cena.mandar(Message::Key(Key::Named(Named::Tab), modifiers));
}

fn digitar(cena: &mut Cena, texto: String) {
    cena.mandar(Message::Typed(texto));
}

#[test]
fn ctrl_l_abre_o_caminho_atual_para_editar() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    assert!(matches!(cena.files.mode, Mode::Location(_)));
    assert_eq!(cena.files.input, cena.work().display().to_string());
}

#[test]
fn enter_abre_a_pasta_digitada_e_entra_no_historico() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    let texto = format!("{}/Fotos/../Fotos", cena.work().display());
    digitar(&mut cena, texto);
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
    assert_eq!(cena.files.mode, Mode::Browse);
    cena.mandar(Message::Key(Key::Named(Named::ArrowLeft), Modifiers::ALT));
    assert_eq!(cena.files.location, cena.work());
}

#[test]
fn caminho_relativo_e_til_partem_da_pasta_e_da_casa() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    digitar(&mut cena, "Fotos".to_owned());
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
    ctrl_l(&mut cena);
    digitar(&mut cena, "~/work".to_owned());
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work());
}

#[test]
fn arquivo_ou_pasta_inexistente_avisam_e_ficam_onde_estavam() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    digitar(&mut cena, "a.txt".to_owned());
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work());
    assert!(cena.files.notice.is_some());
    ctrl_l(&mut cena);
    digitar(&mut cena, "/nao/existe/mesmo".to_owned());
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work());
    assert!(cena.files.notice.is_some());
}

#[test]
fn tab_completa_a_unica_pasta_e_adiciona_a_barra() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    digitar(&mut cena, "~/work/fo".to_owned());
    tab(&mut cena, Modifiers::empty());
    assert_eq!(cena.files.input, "~/work/Fotos/");
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
}

#[test]
fn tab_alterna_entre_varias_e_shift_tab_volta() {
    let mut cena = Cena::nova();
    for nome in ["Música", "Mapas"] {
        assert!(fs::create_dir(cena.work().join(nome)).is_ok());
    }
    ctrl_l(&mut cena);
    let base = format!("{}/", cena.work().display());
    digitar(&mut cena, format!("{base}m"));
    tab(&mut cena, Modifiers::empty());
    assert_eq!(cena.files.input, format!("{base}Mapas"));
    tab(&mut cena, Modifiers::empty());
    assert_eq!(cena.files.input, format!("{base}Música"));
    tab(&mut cena, Modifiers::SHIFT);
    assert_eq!(cena.files.input, format!("{base}Mapas"));
    digitar(&mut cena, format!("{base}Mú"));
    tab(&mut cena, Modifiers::empty());
    assert_eq!(cena.files.input, format!("{base}Música/"));
}

#[test]
fn enter_em_caminho_inexistente_usa_a_primeira_completacao() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    let texto = format!("{}/fot", cena.work().display());
    digitar(&mut cena, texto);
    cena.mandar(Message::Submit);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
}

#[test]
fn esc_cancela_sem_navegar() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    digitar(&mut cena, "Fotos".to_owned());
    cena.mandar(Message::Escape);
    assert_eq!(cena.files.mode, Mode::Browse);
    assert_eq!(cena.files.location, cena.work());
    assert!(cena.files.input.is_empty());
}

#[test]
fn menu_da_area_vazia_edita_o_local() {
    let mut cena = Cena::nova();
    let rows = cena.files.menu_rows(Target::Background);
    assert!(rows.iter().any(|row| row.offers(Action::EditLocation)));
    cena.mandar(Message::ContextBackground);
    cena.mandar(Message::ContextAction(Action::EditLocation));
    assert!(matches!(cena.files.mode, Mode::Location(_)));
}

#[test]
fn uri_de_rede_carrega_a_pagina_de_rede() {
    let mut cena = Cena::nova();
    ctrl_l(&mut cena);
    digitar(&mut cena, "smb://nas/fotos".to_owned());
    cena.mandar(Message::Submit);
    assert_eq!(
        cena.files.network_pending.as_deref(),
        Some("smb://nas/fotos")
    );
}
