use crate::domain::app::{App, Runs};
use crate::domain::category::Category;
use crate::domain::groups::{Group, Groups};
use crate::domain::library::Library;
use crate::domain::placements::{Choice, Placements};
use crate::domain::shelf::{Shelf, after_removal, neighbour, shelves};

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
        groups: Groups::new(vec![Group {
            name: "Favoritos".to_owned(),
            apps: vec!["kitty".to_owned()],
        }]),
        placements: Placements::default(),
    }
}

#[test]
fn a_barra_mostra_inicio_categorias_com_apps_e_os_grupos() {
    let apps = [app("steam", &["Game"]), app("kitty", &["System"])];
    assert_eq!(
        shelves(&apps, &biblioteca()),
        vec![
            Shelf::Home,
            Shelf::Category(Category::System),
            Shelf::Category(Category::Games),
            Shelf::Group(0),
        ]
    );
}

#[test]
fn o_inicio_mostra_so_quem_nao_tem_categoria_nem_grupo() {
    let livre = app("emojify", &["Network"]);
    let jogo = app("steam", &["Game"]);
    let agrupado = app("kitty", &[]);
    assert!(Shelf::Home.holds(&livre, &biblioteca()));
    assert!(!Shelf::Home.holds(&jogo, &biblioteca()));
    assert!(!Shelf::Home.holds(&agrupado, &biblioteca()));
}

#[test]
fn cada_prateleira_mostra_os_seus_aplicativos() {
    let steam = app("steam", &["Game"]);
    let kitty = app("kitty", &["System"]);
    assert!(Shelf::Category(Category::Games).holds(&steam, &biblioteca()));
    assert!(!Shelf::Category(Category::Games).holds(&kitty, &biblioteca()));
    assert!(Shelf::Category(Category::System).holds(&kitty, &biblioteca()));
    assert!(Shelf::Group(0).holds(&kitty, &biblioteca()));
    assert!(!Shelf::Group(0).holds(&steam, &biblioteca()));
}

#[test]
fn escolha_manual_move_o_app_entre_o_inicio_e_a_categoria() {
    let mut library = biblioteca();
    let emojify = app("emojify", &[]);
    library
        .placements
        .place(Category::Utilities, &emojify, Choice::Added);
    assert!(!Shelf::Home.holds(&emojify, &library));
    assert!(Shelf::Category(Category::Utilities).holds(&emojify, &library));
    let steam = app("steam", &["Game"]);
    library
        .placements
        .place(Category::Games, &steam, Choice::Removed);
    assert!(Shelf::Home.holds(&steam, &library));
}

#[test]
fn trocar_de_prateleira_da_a_volta_nas_pontas() {
    let barra = [
        Shelf::Home,
        Shelf::Category(Category::Games),
        Shelf::Group(0),
    ];
    assert_eq!(
        neighbour(&barra, Shelf::Home, 1),
        Shelf::Category(Category::Games)
    );
    assert_eq!(neighbour(&barra, Shelf::Home, -1), Shelf::Group(0));
    assert_eq!(neighbour(&barra, Shelf::Group(0), 1), Shelf::Home);
    assert_eq!(neighbour(&barra, Shelf::Group(4), 1), Shelf::Home);
}

#[test]
fn apagar_um_grupo_ajusta_a_prateleira_aberta() {
    assert_eq!(after_removal(Shelf::Group(1), 1), Shelf::Home);
    assert_eq!(after_removal(Shelf::Group(2), 0), Shelf::Group(1));
    assert_eq!(after_removal(Shelf::Group(0), 3), Shelf::Group(0));
    assert_eq!(after_removal(Shelf::Home, 0), Shelf::Home);
}
