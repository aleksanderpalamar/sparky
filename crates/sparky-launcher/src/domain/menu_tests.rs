use crate::domain::app::{App, Runs};
use crate::domain::category::Category;
use crate::domain::groups::{Group, Groups};
use crate::domain::library::Library;
use crate::domain::menu::{Spot, Target, items, placed};
use crate::domain::shelf::Shelf;

fn app(id: &str, categorias: &[&str]) -> App {
    App {
        id: id.to_owned(),
        name: id.to_owned(),
        detail: None,
        keywords: Vec::new(),
        command: id.to_owned(),
        runs: Runs::Directly,
        icon: None,
        categories: categorias.iter().map(|name| (*name).to_owned()).collect(),
    }
}

fn biblioteca() -> Library {
    Library {
        groups: Groups::new(vec![
            Group {
                name: "Jogos".to_owned(),
                apps: vec!["steam".to_owned()],
            },
            Group {
                name: "Trabalho".to_owned(),
                apps: Vec::new(),
            },
        ]),
        ..Library::default()
    }
}

fn rotulos(target: Target, shelf: Shelf) -> Vec<(&'static str, Option<Spot>)> {
    items(&target, &biblioteca(), shelf)
        .into_iter()
        .map(|item| (item.label, item.spot))
        .collect()
}

#[test]
fn no_inicio_oferece_as_categorias_e_os_grupos_onde_o_app_ainda_nao_esta() {
    let lista = rotulos(Target::App(app("steam", &["Game"])), Shelf::Home);
    assert_eq!(
        lista,
        vec![
            ("menu-add-to", Some(Spot::Category(Category::Office))),
            ("menu-add-to", Some(Spot::Category(Category::Development))),
            ("menu-add-to", Some(Spot::Category(Category::System))),
            ("menu-add-to", Some(Spot::Category(Category::Utilities))),
            ("menu-add-to", Some(Spot::Group(1))),
            ("menu-new-group-with", None),
        ]
    );
}

#[test]
fn dentro_da_categoria_ou_do_grupo_oferece_tirar_primeiro() {
    let steam = app("steam", &["Game"]);
    assert_eq!(
        rotulos(Target::App(steam.clone()), Shelf::Category(Category::Games))[0],
        ("menu-take-from", Some(Spot::Category(Category::Games)))
    );
    assert_eq!(
        rotulos(Target::App(steam), Shelf::Group(0))[0],
        ("menu-take-from", Some(Spot::Group(0)))
    );
}

#[test]
fn no_grupo_oferece_renomear_e_apagar() {
    let lista = rotulos(Target::Group(1), Shelf::Home);
    assert_eq!(
        lista,
        vec![("menu-rename-group", None), ("menu-remove-group", None)]
    );
    assert!(rotulos(Target::Group(9), Shelf::Home).is_empty());
}

#[test]
fn o_menu_nao_sai_da_janela() {
    assert_eq!(
        placed((10.0, 20.0), (800.0, 600.0), (200.0, 100.0)),
        (10.0, 20.0)
    );
    assert_eq!(
        placed((750.0, 580.0), (800.0, 600.0), (200.0, 100.0)),
        (600.0, 500.0)
    );
    assert_eq!(
        placed((50.0, 50.0), (100.0, 80.0), (200.0, 100.0)),
        (0.0, 0.0)
    );
}
