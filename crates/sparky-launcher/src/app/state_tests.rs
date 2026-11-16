use crate::app::test_support::{Memory, app, launcher};
use crate::domain::category::Category;
use crate::domain::grid::Move;
use crate::domain::shelf::Shelf;

fn nomes(launcher: &crate::app::state::Launcher) -> Vec<String> {
    launcher
        .results()
        .iter()
        .map(|app| app.name.clone())
        .collect()
}

#[test]
fn digitar_filtra_e_volta_a_selecao_para_o_primeiro() {
    let mut launcher = launcher(&["Firefox", "Fonts", "Kitty"], &Memory::default());
    launcher.step(Move::Next);
    launcher.step(Move::Next);
    launcher.type_query("fo".to_owned());
    assert_eq!(launcher.cursor, 0);
    assert_eq!(
        launcher.chosen().map(|app| app.name),
        Some("Fonts".to_owned())
    );
}

#[test]
fn andar_para_nas_pontas_da_grade() {
    let mut launcher = launcher(&["A", "B", "C"], &Memory::default());
    launcher.step(Move::Previous);
    assert_eq!(launcher.cursor, 0);
    launcher.step(Move::Down);
    assert_eq!(launcher.cursor, 0);
    for _ in 0..5 {
        launcher.step(Move::Next);
    }
    assert_eq!(launcher.chosen().map(|app| app.name), Some("C".to_owned()));
}

#[test]
fn a_grade_rola_por_linhas_para_mostrar_a_selecao() {
    let nomes: Vec<String> = (0..40).map(|index| format!("App {index:02}")).collect();
    let referencias: Vec<&str> = nomes.iter().map(String::as_str).collect();
    let mut launcher = launcher(&referencias, &Memory::default());
    for _ in 0..5 {
        launcher.step(Move::Down);
    }
    assert_eq!(launcher.cursor, 30);
    assert_eq!(launcher.list.top(), 2);
    assert_eq!(
        launcher.pick(30).map(|app| app.name),
        Some("App 30".to_owned())
    );
}

#[test]
fn sem_resultado_nao_ha_escolha() {
    let mut launcher = launcher(&["Firefox"], &Memory::default());
    launcher.type_query("zzz".to_owned());
    assert_eq!(launcher.chosen(), None);
}

#[test]
fn a_categoria_mostra_so_os_seus_e_a_busca_procura_em_todos() {
    let mut launcher = launcher(&[], &Memory::default());
    launcher.apps = vec![app("Steam", &["Game"]), app("Kitty", &["System"])];
    launcher.show_shelf(Shelf::Category(Category::Games));
    assert_eq!(nomes(&launcher), vec!["Steam"]);
    launcher.type_query("kit".to_owned());
    assert_eq!(nomes(&launcher), vec!["Kitty"]);
}

#[test]
fn trocar_de_prateleira_limpa_a_busca_e_volta_ao_inicio() {
    let mut launcher = launcher(&[], &Memory::default());
    launcher.apps = vec![app("Steam", &["Game"]), app("Kitty", &[])];
    launcher.type_query("kit".to_owned());
    launcher.next_shelf(1);
    assert_eq!(launcher.shelf, Shelf::Category(Category::Games));
    assert!(launcher.query.is_empty());
    launcher.next_shelf(1);
    assert_eq!(launcher.shelf, Shelf::Home);
}

#[test]
fn o_menu_do_sistema_nao_tem_prateleiras() {
    let mut launcher = launcher(&["A"], &Memory::default());
    launcher.mode = crate::domain::mode::Mode::System;
    assert!(launcher.shelves().is_empty());
}
