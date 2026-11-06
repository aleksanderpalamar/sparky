use std::fs;

use super::test_scene::Cena;
use crate::domain::sidebar_location::SidebarLocation;

#[test]
fn favoritos_abrem_arquivo_e_navegacao_local_limpa_pagina_virtual() {
    let mut cena = Cena::nova();
    let file = cena.work().join("a.txt");
    cena.files.services.sidebar.favorite(&file, true).unwrap();
    cena.files.open_sidebar(SidebarLocation::Favorites);
    assert_eq!(cena.files.sidebar_items.len(), 1);
    cena.files.open_sidebar_item(0);
    assert_eq!(cena.files.sidebar_page, Some(SidebarLocation::Favorites));
    cena.files.home();
    assert_eq!(cena.files.sidebar_page, None);
}

#[test]
fn mover_favorito_mantem_referencia_no_destino() {
    let mut cena = Cena::nova();
    let file = cena.work().join("a.txt");
    let destination = cena.work().join("Fotos");
    cena.files.services.sidebar.favorite(&file, true).unwrap();
    cena.files.listing.select_name("a.txt");
    cena.files.cut();
    assert!(cena.files.go(destination.clone()));
    cena.files.paste();
    assert!(fs::metadata(destination.join("a.txt")).is_ok());
    assert!(
        cena.files
            .services
            .sidebar
            .is_favorite(&destination.join("a.txt"))
    );
    assert!(!cena.files.services.sidebar.is_favorite(&file));
}
