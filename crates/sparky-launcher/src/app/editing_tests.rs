use crate::app::editing::Focus;
use crate::app::state::Launcher;
use crate::app::test_support::{Memory, app, launcher};
use crate::domain::category::Category;
use crate::domain::groups::{Group, Groups};
use crate::domain::library::Library;
use crate::domain::menu::{Action, Spot, Target};
use crate::domain::overlay::Overlay;
use crate::domain::shelf::Shelf;

fn grupo(name: &str, apps: &[&str]) -> Group {
    Group {
        name: name.to_owned(),
        apps: apps.iter().map(|app| (*app).to_owned()).collect(),
    }
}

fn com_grupos(lista: Vec<Group>) -> Memory {
    let memoria = Memory::default();
    memoria.0.borrow_mut().groups = Groups::new(lista);
    memoria
}

fn salvos(memoria: &Memory) -> Vec<Group> {
    memoria.0.borrow().groups.list().to_vec()
}

fn escolher(launcher: &mut Launcher, wanted: impl Fn(&Action) -> bool) -> Focus {
    let index = launcher
        .menu_items()
        .iter()
        .position(|item| wanted(&item.action));
    let Some(index) = index else {
        panic!("o menu deveria ter a ação");
    };
    launcher.choose(index)
}

fn nomes(launcher: &Launcher) -> Vec<String> {
    launcher
        .results()
        .iter()
        .map(|app| app.name.clone())
        .collect()
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
    assert_eq!(salvos(&memoria), vec![grupo("Trabalho", &[])]);
}

#[test]
fn nome_repetido_fica_recusado_ate_mudar_o_texto() {
    let memoria = com_grupos(vec![grupo("Jogos", &[])]);
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
fn adicionar_ao_grupo_tira_do_inicio_e_tirar_do_grupo_devolve() {
    let memoria = com_grupos(vec![grupo("Jogos", &[])]);
    let mut launcher = launcher(&["Kitty", "Steam"], &memoria);
    launcher.open_app_menu("steam");
    escolher(&mut launcher, |action| {
        matches!(
            action,
            Action::AddTo {
                spot: Spot::Group(0),
                ..
            }
        )
    });
    assert_eq!(salvos(&memoria), vec![grupo("Jogos", &["steam"])]);
    assert_eq!(nomes(&launcher), vec!["Kitty"]);
    launcher.show_shelf(Shelf::Group(0));
    launcher.open_app_menu("steam");
    escolher(&mut launcher, |action| {
        matches!(action, Action::TakeFrom { .. })
    });
    assert_eq!(salvos(&memoria), vec![grupo("Jogos", &[])]);
    launcher.show_shelf(Shelf::Home);
    assert_eq!(nomes(&launcher), vec!["Kitty", "Steam"]);
}

#[test]
fn adicionar_a_categoria_tira_do_inicio_e_fica_salvo() {
    let memoria = Memory::default();
    let mut launcher = launcher(&["Emojify", "Kitty"], &memoria);
    launcher.open_app_menu("emojify");
    escolher(&mut launcher, |action| {
        matches!(
            action,
            Action::AddTo {
                spot: Spot::Category(Category::Utilities),
                ..
            }
        )
    });
    assert_eq!(nomes(&launcher), vec!["Kitty"]);
    assert!(
        launcher
            .shelves()
            .contains(&Shelf::Category(Category::Utilities))
    );
    let salvo = memoria.0.borrow().clone();
    assert_eq!(salvo.placements.list().len(), 1);
}

#[test]
fn tirar_da_categoria_do_desktop_devolve_ao_inicio() {
    let memoria = Memory::default();
    let mut launcher = launcher(&[], &memoria);
    launcher.apps = vec![app("Steam", &["Game"]), app("Kitty", &[])];
    assert_eq!(nomes(&launcher), vec!["Kitty"]);
    launcher.show_shelf(Shelf::Category(Category::Games));
    launcher.open_app_menu("steam");
    escolher(&mut launcher, |action| {
        matches!(action, Action::TakeFrom { .. })
    });
    launcher.show_shelf(Shelf::Home);
    assert_eq!(nomes(&launcher), vec!["Kitty", "Steam"]);
}

#[test]
fn novo_grupo_pelo_menu_ja_leva_o_aplicativo() {
    let memoria = Memory::default();
    let mut launcher = launcher(&["Steam"], &memoria);
    launcher.open_app_menu("steam");
    let foco = escolher(&mut launcher, |action| {
        matches!(action, Action::NewGroupWith(_))
    });
    assert_eq!(foco, Focus::Name);
    launcher.type_name("Jogos".to_owned());
    launcher.confirm_name();
    assert_eq!(salvos(&memoria), vec![grupo("Jogos", &["steam"])]);
}

#[test]
fn renomear_comeca_com_o_nome_atual_e_apagar_devolve_os_apps() {
    let memoria = com_grupos(vec![grupo("Jogos", &[]), grupo("Dev", &["steam"])]);
    let mut launcher = launcher(&["Steam"], &memoria);
    assert!(nomes(&launcher).is_empty());
    launcher.open_menu(Target::Group(1));
    escolher(&mut launcher, |action| matches!(action, Action::Rename(1)));
    assert!(matches!(&launcher.overlay, Overlay::Naming { text, .. } if text == "Dev"));
    launcher.dismiss();
    launcher.show_shelf(Shelf::Group(1));
    launcher.open_menu(Target::Group(1));
    escolher(&mut launcher, |action| matches!(action, Action::Remove(1)));
    assert_eq!(launcher.shelf, Shelf::Home);
    assert_eq!(salvos(&memoria), vec![grupo("Jogos", &[])]);
    assert_eq!(nomes(&launcher), vec!["Steam"]);
}

#[test]
fn o_cursor_do_menu_para_nas_pontas() {
    let mut launcher = launcher(&["Steam"], &Memory::default());
    launcher.open_menu_on_chosen();
    assert_eq!(launcher.overlay, Overlay::Closed);
    launcher.hover(0);
    launcher.open_menu_on_chosen();
    let ultimo = launcher.menu_items().len() - 1;
    launcher.menu_step(50);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == ultimo));
    launcher.menu_step(-90);
    assert!(matches!(&launcher.overlay, Overlay::Menu(menu) if menu.cursor == 0));
}

#[test]
fn a_biblioteca_salva_comeca_vazia_sem_arquivo() {
    assert_eq!(*Memory::default().0.borrow(), Library::default());
}
