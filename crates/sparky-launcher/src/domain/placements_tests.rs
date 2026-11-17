use crate::domain::app::{App, Runs};
use crate::domain::category::Category;
use crate::domain::placements::{Choice, Placement, Placements};

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

#[test]
fn sem_escolha_vale_a_categoria_do_desktop() {
    let lista = Placements::default();
    assert!(lista.holds(Category::Games, &app("steam", &["Game"])));
    assert!(!lista.holds(Category::Games, &app("kitty", &[])));
}

#[test]
fn adicionar_a_mao_poe_o_aplicativo_na_categoria_e_tirar_devolve() {
    let mut lista = Placements::default();
    let kitty = app("kitty", &[]);
    assert!(lista.place(Category::System, &kitty, Choice::Added));
    assert!(lista.holds(Category::System, &kitty));
    assert!(!lista.place(Category::System, &kitty, Choice::Added));
    assert!(lista.place(Category::System, &kitty, Choice::Removed));
    assert!(!lista.holds(Category::System, &kitty));
    assert!(lista.list().is_empty());
}

#[test]
fn tirar_um_app_da_categoria_do_desktop_fica_guardado() {
    let mut lista = Placements::default();
    let steam = app("steam", &["Game"]);
    assert!(lista.place(Category::Games, &steam, Choice::Removed));
    assert!(!lista.holds(Category::Games, &steam));
    assert_eq!(
        lista.list(),
        &[Placement {
            category: Category::Games,
            app: "steam".to_owned(),
            choice: Choice::Removed,
        }]
    );
}

#[test]
fn ao_carregar_a_ultima_escolha_vence() {
    let escolha = |choice| Placement {
        category: Category::Office,
        app: "x".to_owned(),
        choice,
    };
    let lista = Placements::new(vec![escolha(Choice::Added), escolha(Choice::Removed)]);
    assert_eq!(lista.list(), &[escolha(Choice::Removed)]);
}
