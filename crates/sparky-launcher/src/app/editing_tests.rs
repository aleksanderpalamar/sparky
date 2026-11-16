use crate::app::editing::Focus;
use crate::app::test_support::{Memory, launcher};
use crate::domain::groups::Group;
use crate::domain::menu::Target;
use crate::domain::overlay::Overlay;
use crate::domain::shelf::Shelf;

fn grupo(name: &str, apps: &[&str]) -> Group {
    Group {
        name: name.to_owned(),
        apps: apps.iter().map(|app| (*app).to_owned()).collect(),
    }
}

#[test]
fn novo_grupo_pede_o_nome_salva_e_abre_o_grupo() {
    let memoria = Memory::default();
    let mut launcher = launcher(&["Kitty"], &memoria);
    assert_eq!(launcher.add_group(), Focus::Name);
    launcher.type_name("Trabalho".to_owned());
    assert_eq!(launcher.confirm_name(), Focus::Search);
    assert_eq!(launcher.overlay, Overlay::Closed);
    assert_eq!(launcher.shelf, Shelf::Group(0));
    assert_eq!(*memoria.0.borrow(), vec![grupo("Trabalho", &[])]);
}

#[test]
fn nome_repetido_fica_recusado_ate_mudar_o_texto() {
    let memoria = Memory::default();
    *memoria.0.borrow_mut() = vec![grupo("Jogos", &[])];
    let mut launcher = launcher(&["Kitty"], &memoria);
    launcher.add_group();
    launcher.type_name("jogos".to_owned());
    assert_eq!(launcher.confirm_name(), Focus::Stay);
    assert!(matches!(
        launcher.overlay,
        Overlay::Naming { refused: true, .. }
    ));
    launcher.type_name("jogos 2".to_owned());
    assert!(matches!(
        launcher.overlay,
        Overlay::Naming { refused: false, .. }
    ));
}

#[test]
fn o_menu_do_aplicativo_adiciona_e_tira_do_grupo() {
    let memoria = Memory::default();
    *memoria.0.borrow_mut() = vec![grupo("Jogos", &[])];
    let mut launcher = launcher(&["Steam"], &memoria);
    launcher.open_menu(Target::App("steam".to_owned()));
    assert_eq!(launcher.choose(0), Focus::Search);
    assert_eq!(*memoria.0.borrow(), vec![grupo("Jogos", &["steam"])]);
    launcher.show_shelf(Shelf::Group(0));
    launcher.open_menu(Target::App("steam".to_owned()));
    launcher.choose(0);
    assert_eq!(*memoria.0.borrow(), vec![grupo("Jogos", &[])]);
    assert_eq!(launcher.chosen(), None);
}

#[test]
fn adicionar_ao_grupo_tira_do_inicio_e_apagar_o_grupo_devolve() {
    let memoria = Memory::default();
    *memoria.0.borrow_mut() = vec![grupo("Jogos", &[])];
    let mut launcher = launcher(&["Kitty", "Steam"], &memoria);
    let inicio = |launcher: &crate::app::state::Launcher| -> Vec<String> {
        launcher
            .results()
            .iter()
            .map(|app| app.name.clone())
            .collect()
    };
    launcher.open_menu(Target::App("steam".to_owned()));
    launcher.choose(0);
    assert_eq!(inicio(&launcher), vec!["Kitty"]);
    launcher.type_query("ste".to_owned());
    assert_eq!(inicio(&launcher), vec!["Steam"]);
    launcher.type_query(String::new());
    launcher.open_menu(Target::Group(0));
    launcher.choose(1);
    assert_eq!(inicio(&launcher), vec!["Kitty", "Steam"]);
}

#[test]
fn novo_grupo_pelo_menu_ja_leva_o_aplicativo() {
    let memoria = Memory::default();
    let mut launcher = launcher(&["Steam"], &memoria);
    launcher.open_menu(Target::App("steam".to_owned()));
    assert_eq!(launcher.choose(0), Focus::Name);
    launcher.type_name("Jogos".to_owned());
    launcher.confirm_name();
    assert_eq!(*memoria.0.borrow(), vec![grupo("Jogos", &["steam"])]);
}

#[test]
fn renomear_comeca_com_o_nome_atual_e_apagar_volta_ao_inicio() {
    let memoria = Memory::default();
    *memoria.0.borrow_mut() = vec![grupo("Jogos", &[]), grupo("Dev", &[])];
    let mut launcher = launcher(&["Steam"], &memoria);
    launcher.open_menu(Target::Group(1));
    launcher.choose(0);
    assert!(matches!(&launcher.overlay, Overlay::Naming { text, .. } if text == "Dev"));
    launcher.dismiss();
    launcher.show_shelf(Shelf::Group(1));
    launcher.open_menu(Target::Group(1));
    launcher.choose(1);
    assert_eq!(launcher.shelf, Shelf::Home);
    assert_eq!(*memoria.0.borrow(), vec![grupo("Jogos", &[])]);
}

#[test]
fn o_cursor_do_menu_para_nas_pontas() {
    let memoria = Memory::default();
    *memoria.0.borrow_mut() = vec![grupo("A", &[])];
    let mut launcher = launcher(&["Steam"], &memoria);
    launcher.open_menu_on_chosen();
    launcher.menu_step(5);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == 1));
    launcher.menu_step(-9);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == 0));
}
