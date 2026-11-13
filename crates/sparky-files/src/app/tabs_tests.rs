use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::entry::{Kind, entry};
use crate::domain::grid::ViewMode;
use crate::domain::order::{Order, SortDirection, SortKey};
use crate::domain::search::SearchEvent;
use crate::view::tabs::titles;

fn ctrl(cena: &mut Cena, letra: &str) {
    cena.mandar(Message::Key(Key::Character(letra.into()), Modifiers::CTRL));
}

fn ctrl_tab(cena: &mut Cena, modifiers: Modifiers) {
    cena.mandar(Message::Key(Key::Named(Named::Tab), modifiers));
}

fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|item| item.name.clone())
        .collect()
}

#[test]
fn ctrl_t_abre_outra_aba_na_mesma_pasta_e_cada_uma_navega_sozinha() {
    let mut cena = Cena::nova();
    ctrl(&mut cena, "t");
    assert_eq!(cena.files.tabs.len(), 2);
    assert_eq!(cena.files.location, cena.work());
    cena.files.go(cena.work().join("Fotos"));
    assert_eq!(titles(&cena.files), ["work", "Fotos"]);
    ctrl_tab(&mut cena, Modifiers::CTRL);
    assert_eq!(cena.files.location, cena.work());
    assert_eq!(nomes(&cena), ["Fotos", "a.txt"]);
    ctrl_tab(&mut cena, Modifiers::CTRL | Modifiers::SHIFT);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
}

#[test]
fn ctrl_w_fecha_a_aba_atual_e_a_ultima_continua_aberta() {
    let mut cena = Cena::nova();
    ctrl(&mut cena, "t");
    cena.files.go(cena.work().join("Fotos"));
    ctrl(&mut cena, "w");
    assert_eq!(cena.files.tabs.len(), 1);
    assert_eq!(cena.files.location, cena.work());
    ctrl(&mut cena, "w");
    assert_eq!(cena.files.tabs.len(), 1);
}

#[test]
fn voltar_para_uma_aba_rele_a_pasta_e_mantem_a_selecao() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    ctrl(&mut cena, "t");
    assert!(fs::write(cena.work().join("b.txt"), "b").is_ok());
    cena.mandar(Message::Tab(0));
    assert_eq!(nomes(&cena), ["Fotos", "a.txt", "b.txt"]);
    assert_eq!(cena.selecionado().as_deref(), Some("a.txt"));
}

#[test]
fn clicar_no_x_de_outra_aba_fecha_so_ela() {
    let mut cena = Cena::nova();
    ctrl(&mut cena, "t");
    cena.files.go(cena.work().join("Fotos"));
    cena.mandar(Message::CloseTab(0));
    assert_eq!(cena.files.tabs.len(), 1);
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
}

#[test]
fn a_busca_de_uma_aba_guardada_continua_chegando_nela() {
    let mut cena = Cena::nova();
    assert!(fs::write(cena.work().join("Fotos/abc.txt"), "x").is_ok());
    ctrl(&mut cena, "f");
    cena.mandar(Message::Typed("a".to_owned()));
    cena.mandar(Message::Submit);
    let Some(id) = cena.files.search.as_ref().map(|busca| busca.id) else {
        panic!("a busca deveria ter começado")
    };
    ctrl(&mut cena, "t");
    assert!(cena.files.search.is_none());
    let achado = entry("Fotos/abc.txt", Kind::File);
    cena.mandar(Message::Search(id, SearchEvent::Found(vec![achado])));
    assert!(nomes(&cena).iter().all(|nome| nome != "Fotos/abc.txt"));
    cena.mandar(Message::Tab(0));
    assert!(nomes(&cena).contains(&"Fotos/abc.txt".to_owned()));
    assert_eq!(titles(&cena.files), ["Busca: a", "work"]);
}

#[test]
fn a_busca_de_uma_aba_guardada_respeita_a_ordem_por_tamanho() {
    let mut cena = Cena::nova();
    let escrito = fs::write(cena.work().join("Fotos/nota a.txt"), "x")
        .and_then(|()| fs::write(cena.work().join("Fotos/nota b.txt"), "x".repeat(300)));
    assert!(escrito.is_ok());
    cena.files
        .arrange(Order::new(SortKey::Size, SortDirection::Descending));
    ctrl(&mut cena, "f");
    cena.mandar(Message::Typed("nota".to_owned()));
    cena.mandar(Message::Submit);
    let Some(id) = cena.files.search.as_ref().map(|busca| busca.id) else {
        panic!("a busca deveria ter começado")
    };
    ctrl(&mut cena, "t");
    let achados = ["Fotos/nota a.txt", "Fotos/nota b.txt"].map(|nome| entry(nome, Kind::File));
    cena.mandar(Message::Search(id, SearchEvent::Found(achados.to_vec())));
    cena.mandar(Message::Tab(0));
    assert_eq!(nomes(&cena), ["Fotos/nota b.txt", "Fotos/nota a.txt"]);
}

#[test]
fn voltar_a_uma_aba_ajusta_a_rolagem_ao_modo_de_exibicao_atual() {
    let mut cena = Cena::nova();
    cena.files.change_view(ViewMode::List);
    ctrl(&mut cena, "t");
    cena.files.change_view(ViewMode::Grid);
    cena.mandar(Message::Tab(0));
    assert_eq!(cena.files.viewport.rows(), cena.files.layout.rows);
}
