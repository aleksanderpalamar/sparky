use crate::app::navigation::Scroll;
use crate::app::test_support::{Memory, app, launcher};
use crate::app::view::{CELL_HEIGHT, ROW_STEP};
use crate::domain::category::Category;
use crate::domain::grid::{Move, Window};
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
    assert_eq!(launcher.cursor, Some(0));
    assert_eq!(
        launcher.chosen().map(|app| app.name),
        Some("Fonts".to_owned())
    );
}

#[test]
fn andar_para_nas_pontas_da_grade() {
    let mut launcher = launcher(&["A", "B", "C"], &Memory::default());
    launcher.step(Move::Previous);
    assert_eq!(launcher.cursor, Some(0));
    launcher.step(Move::Down);
    assert_eq!(launcher.cursor, Some(0));
    for _ in 0..5 {
        launcher.step(Move::Next);
    }
    assert_eq!(launcher.chosen().map(|app| app.name), Some("C".to_owned()));
}

#[test]
fn descer_alem_da_janela_pede_para_rolar_ate_a_linha() {
    let nomes: Vec<String> = (0..40).map(|index| format!("App {index:02}")).collect();
    let referencias: Vec<&str> = nomes.iter().map(String::as_str).collect();
    let mut launcher = launcher(&referencias, &Memory::default());
    launcher.scrolled(Window {
        offset: 0.0,
        height: 440.0,
    });
    launcher.hover(0);
    assert_eq!(launcher.step(Move::Down), Scroll::Keep);
    for _ in 0..3 {
        launcher.step(Move::Down);
    }
    assert_eq!(launcher.cursor, Some(24));
    assert_eq!(
        launcher.step(Move::Down),
        Scroll::To(5.0 * ROW_STEP + CELL_HEIGHT - 440.0)
    );
    assert_eq!(
        launcher.pick(30).map(|app| app.name),
        Some("App 30".to_owned())
    );
}

#[test]
fn buscar_ou_trocar_de_prateleira_volta_ao_topo() {
    let mut launcher = launcher(&["A", "B"], &Memory::default());
    launcher.scrolled(Window {
        offset: 300.0,
        height: 440.0,
    });
    assert_eq!(launcher.type_query("a".to_owned()), Scroll::To(0.0));
    assert_eq!(launcher.window.offset, 0.0);
    launcher.scrolled(Window {
        offset: 200.0,
        height: 440.0,
    });
    assert_eq!(launcher.next_shelf(1), Scroll::To(0.0));
}

#[test]
fn ao_abrir_nada_vem_selecionado_e_a_primeira_seta_marca_o_primeiro() {
    let mut launcher = launcher(&["A", "B"], &Memory::default());
    assert_eq!(launcher.cursor, None);
    assert_eq!(launcher.chosen(), None);
    launcher.step(Move::Down);
    assert_eq!(launcher.cursor, Some(0));
}

#[test]
fn digitar_marca_o_primeiro_e_apagar_a_busca_desmarca() {
    let mut launcher = launcher(&["Alfa", "Beta"], &Memory::default());
    launcher.type_query("be".to_owned());
    assert_eq!(
        launcher.chosen().map(|app| app.name),
        Some("Beta".to_owned())
    );
    launcher.type_query(String::new());
    assert_eq!(launcher.cursor, None);
}

#[test]
fn trocar_de_prateleira_tira_a_selecao() {
    let mut launcher = launcher(&["A"], &Memory::default());
    launcher.hover(0);
    launcher.show_shelf(Shelf::Home);
    assert_eq!(launcher.cursor, None);
}

#[test]
fn o_menu_do_sistema_abre_com_a_primeira_opcao_marcada() {
    let lancador = crate::app::state::Launcher::new(
        crate::domain::system::entries(),
        sparky_theme::Skin::default(),
        crate::domain::mode::Mode::System,
        Box::new(Memory::default()),
    );
    assert_eq!(lancador.cursor, Some(0));
}

#[test]
fn sem_resultado_nao_ha_escolha() {
    let mut launcher = launcher(&["Firefox"], &Memory::default());
    launcher.type_query("zzz".to_owned());
    assert_eq!(launcher.chosen(), None);
}

#[test]
fn o_inicio_mostra_so_quem_nao_tem_categoria() {
    let mut launcher = launcher(&[], &Memory::default());
    launcher.apps = vec![
        app("Steam", &["Game"]),
        app("Emojify", &["Network"]),
        app("Kitty", &["System"]),
    ];
    assert_eq!(nomes(&launcher), vec!["Emojify"]);
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
