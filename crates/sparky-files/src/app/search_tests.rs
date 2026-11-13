use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use crate::domain::order::{Order, SortDirection, SortKey};
use crate::domain::search::{Finish, SearchEvent};

fn com_arvore() -> Cena {
    let cena = Cena::nova();
    let raiz = cena.work();
    let criado = fs::create_dir_all(raiz.join("Fotos/Praia"))
        .and_then(|()| fs::write(raiz.join("Fotos/Praia/Fóto 1.jpg"), "x"))
        .and_then(|()| fs::write(raiz.join("Fotos/nota.txt"), "x"));
    assert!(criado.is_ok());
    cena
}

fn buscar(cena: &mut Cena, texto: &str) {
    cena.mandar(Message::Key(Key::Character("f".into()), Modifiers::CTRL));
    assert_eq!(cena.files.mode, Mode::Search);
    cena.mandar(Message::Typed(texto.to_owned()));
    cena.concluir(Message::Submit);
}

fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|entry| entry.name.clone())
        .collect()
}

#[test]
fn ctrl_f_busca_nas_subpastas_e_mostra_o_caminho_relativo() {
    let mut cena = com_arvore();
    buscar(&mut cena, "foto");
    assert_eq!(nomes(&cena), ["Fotos", "Fotos/Praia/Fóto 1.jpg"]);
    assert_eq!(cena.files.location, cena.work());
    let busca = cena.files.search.as_ref().map(|busca| busca.finish);
    assert_eq!(busca, Some(Some(Finish::Complete)));
}

#[test]
fn esc_volta_para_a_pasta_e_reler_nao_desfaz_os_resultados() {
    let mut cena = com_arvore();
    buscar(&mut cena, "foto");
    cena.mandar(Message::FolderChanged);
    assert_eq!(nomes(&cena).len(), 2);
    cena.mandar(Message::Escape);
    assert!(cena.files.search.is_none());
    assert_eq!(nomes(&cena), ["Fotos", "a.txt"]);
}

#[test]
fn abrir_uma_pasta_do_resultado_encerra_a_busca() {
    let mut cena = com_arvore();
    buscar(&mut cena, "praia");
    assert_eq!(nomes(&cena), ["Fotos/Praia"]);
    cena.nomeada(Named::Enter);
    assert!(cena.files.search.is_none());
    assert_eq!(cena.files.location, cena.work().join("Fotos/Praia"));
}

#[test]
fn mandar_um_resultado_para_a_lixeira_tira_ele_da_lista() {
    let mut cena = com_arvore();
    buscar(&mut cena, "foto 1");
    cena.tecla("d");
    assert!(!cena.work().join("Fotos/Praia/Fóto 1.jpg").exists());
    assert!(nomes(&cena).is_empty());
    assert!(cena.files.search.is_some());
}

#[test]
fn renomear_e_criar_ficam_bloqueados_nos_resultados() {
    let mut cena = com_arvore();
    buscar(&mut cena, "nota");
    cena.tecla("r");
    cena.tecla("a");
    assert_eq!(cena.files.mode, Mode::Browse);
}

#[test]
fn resultados_de_uma_busca_antiga_sao_ignorados() {
    let mut cena = com_arvore();
    buscar(&mut cena, "nota");
    let antiga = cena.files.search.as_ref().map_or(0, |busca| busca.id) + 99;
    let falso = crate::domain::entry::entry("intruso", crate::domain::entry::Kind::File);
    cena.mandar(Message::Search(antiga, SearchEvent::Found(vec![falso])));
    assert_eq!(nomes(&cena), ["Fotos/nota.txt"]);
}

#[test]
fn cabecalho_e_barra_de_status_descrevem_a_busca() {
    let mut cena = com_arvore();
    buscar(&mut cena, "foto");
    let Some(busca) = cena.files.search.clone() else {
        panic!("a busca deveria estar ativa")
    };
    assert_eq!(
        crate::view::search_status::title(&cena.files, &busca),
        "Busca por “foto” em ~/work"
    );
    assert_eq!(
        crate::view::search_status::summary(&cena.files, &busca),
        "2 resultados · Esc volta"
    );
}

#[test]
fn resultados_respeitam_a_ordem_por_tamanho_escolhida_antes_da_busca() {
    let mut cena = Cena::nova();
    let criado = fs::create_dir_all(cena.work().join("sub"))
        .and_then(|()| fs::write(cena.work().join("nota a.txt"), "x"))
        .and_then(|()| fs::write(cena.work().join("sub/nota b.txt"), "x".repeat(500)))
        .and_then(|()| fs::write(cena.work().join("nota c.txt"), "x".repeat(50)));
    assert!(criado.is_ok());
    cena.files
        .arrange(Order::new(SortKey::Size, SortDirection::Descending));
    buscar(&mut cena, "nota");
    assert_eq!(nomes(&cena), ["sub/nota b.txt", "nota c.txt", "nota a.txt"]);
}
