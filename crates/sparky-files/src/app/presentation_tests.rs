use std::fs;

use iced::Size;
use iced::keyboard::{Key, Modifiers, key::Named};

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::grid::ViewMode;

fn many() -> Cena {
    let mut scene = Cena::nova();
    for index in 0..40 {
        fs::write(scene.work().join(format!("arquivo-{index:02}.txt")), "").unwrap();
    }
    scene.files.refresh();
    scene.mandar(Message::Resized(Size::new(650.0, 380.0)));
    scene
}

#[test]
fn alternar_grade_e_lista_preserva_filtro_selecao_e_marcacoes() {
    let mut scene = many();
    scene.files.listing.set_filter("arquivo");
    scene.files.listing.select(5);
    scene.files.listing.toggle_mark();
    let selected = scene.selecionado();
    scene.mandar(Message::View(ViewMode::List));
    assert_eq!(scene.files.layout.columns, 1);
    scene.mandar(Message::View(ViewMode::Grid));
    assert!(scene.files.layout.columns > 1);
    assert_eq!(scene.selecionado(), selected);
    assert_eq!(scene.files.listing.filter(), "arquivo");
    assert_eq!(scene.files.listing.marked_count(), 1);
}

#[test]
fn setas_movem_por_celulas_e_h_l_continuam_entrando_e_voltando() {
    let mut scene = many();
    let columns = scene.files.layout.columns;
    scene.mandar(Message::Key(
        Key::Named(Named::ArrowDown),
        Modifiers::empty(),
    ));
    assert_eq!(scene.files.listing.cursor(), columns);
    scene.mandar(Message::Key(
        Key::Named(Named::ArrowRight),
        Modifiers::empty(),
    ));
    assert_eq!(scene.files.listing.cursor(), columns + 1);
    scene.files.listing.first();
    scene.mandar(Message::Key(Key::Character("l".into()), Modifiers::empty()));
    assert_eq!(scene.files.location, scene.work().join("Fotos"));
    scene.mandar(Message::Key(Key::Character("h".into()), Modifiers::empty()));
    assert_eq!(scene.files.location, scene.work());
}

#[test]
fn resize_mantem_selecao_visivel_em_linhas_completas() {
    let mut scene = many();
    scene.files.listing.select(30);
    scene.files.follow();
    let selected = scene.selecionado();
    scene.mandar(Message::Resized(Size::new(420.0, 270.0)));
    assert_eq!(scene.selecionado(), selected);
    assert!(
        scene
            .files
            .layout
            .range(scene.files.viewport.top(), scene.files.listing.len())
            .contains(&30)
    );
}

#[test]
fn atualizacao_periodica_nao_desfaz_rolagem_manual() {
    let mut scene = many();
    scene.mandar(Message::Scrolled(3));
    let top = scene.files.viewport.top();
    assert!(top > 0);
    scene.mandar(Message::Tick);
    assert_eq!(scene.files.viewport.top(), top);
}

#[test]
fn clique_usa_indice_absoluto_depois_da_rolagem() {
    let mut scene = many();
    scene.mandar(Message::Scrolled(3));
    let index = scene
        .files
        .layout
        .range(scene.files.viewport.top(), scene.files.listing.len())
        .start;
    scene.mandar(Message::Clicked(index));
    assert_eq!(scene.files.listing.cursor(), index);
}

#[test]
fn ativacao_invalida_nao_abre_a_pasta_selecionada() {
    let mut scene = Cena::nova();
    let location = scene.files.location.clone();
    scene.mandar(Message::Activated(99));
    assert_eq!(scene.files.location, location);
}

#[test]
fn atualizacao_de_icones_preserva_edicao_e_selecao() {
    struct NewIcons;
    impl crate::ports::IconSource for NewIcons {
        fn load(&mut self) -> crate::domain::icons::Icons {
            [(
                crate::domain::icons::IconKind::Folder,
                "/tema/folder.svg".into(),
            )]
            .into_iter()
            .collect()
        }
    }
    let mut scene = Cena::nova();
    scene.mandar(Message::Key(Key::Character("r".into()), Modifiers::empty()));
    scene.mandar(Message::Typed("Novo nome".to_owned()));
    let mode = scene.files.mode.clone();
    let selected = scene.selecionado();
    scene.files.services.icons = Box::new(NewIcons);
    scene.mandar(Message::Tick);
    assert_eq!(scene.files.mode, mode);
    assert_eq!(scene.files.input, "Novo nome");
    assert_eq!(scene.selecionado(), selected);
    assert_eq!(
        scene
            .files
            .icons
            .get(&crate::domain::icons::IconKind::Folder),
        Some(&std::path::PathBuf::from("/tema/folder.svg"))
    );
}

#[test]
fn renomear_para_fora_da_pagina_mantem_item_visivel() {
    let mut scene = many();
    scene.mandar(Message::View(ViewMode::List));
    scene.files.listing.select(2);
    scene.files.begin_rename();
    scene.mandar(Message::Typed("z-ultimo.txt".to_owned()));
    scene.mandar(Message::Submit);
    assert_eq!(scene.selecionado().as_deref(), Some("z-ultimo.txt"));
    assert!(
        scene
            .files
            .layout
            .range(scene.files.viewport.top(), scene.files.listing.len())
            .contains(&scene.files.listing.cursor())
    );
}

#[test]
fn criar_fora_da_pagina_mantem_item_visivel() {
    let mut scene = many();
    scene.mandar(Message::View(ViewMode::List));
    scene.files.begin_create();
    scene.mandar(Message::Typed("z-novo.txt".to_owned()));
    scene.mandar(Message::Submit);
    assert_eq!(scene.selecionado().as_deref(), Some("z-novo.txt"));
    assert!(
        scene
            .files
            .layout
            .range(scene.files.viewport.top(), scene.files.listing.len())
            .contains(&scene.files.listing.cursor())
    );
}
