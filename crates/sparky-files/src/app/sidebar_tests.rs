use std::fs;

use gio::prelude::*;
use iced::Task;
use iced::futures::StreamExt;
use iced_runtime::Action;

use super::test_scene::Cena;
use super::update::update;
use crate::app::message::Message;
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation, SidebarTarget};

fn complete(scene: &mut Cena, task: Task<Message>) {
    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return;
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        while let Some(action) = stream.next().await {
            let Action::Output(message) = action else {
                continue;
            };
            let _ = update(&mut scene.files, message);
        }
    });
}

#[test]
fn favoritos_abrem_arquivo_e_navegacao_local_limpa_pagina_virtual() {
    let mut cena = Cena::nova();
    let file = cena.work().join("a.txt");
    cena.files.services.sidebar.favorite(&file, true).unwrap();
    cena.files.open_sidebar(SidebarLocation::Favorites);
    assert_eq!(cena.files.sidebar_items.len(), 1);
    let _ = cena.files.open_sidebar_item(0);
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

#[test]
fn consulta_de_rede_roda_fora_da_thread_da_interface() {
    let mut cena = Cena::nova();
    let folder = cena.work().join("Fotos");
    fs::write(folder.join("imagem.txt"), "dados").unwrap();
    let uri = gio::File::for_path(&folder).uri().to_string();
    cena.files.sidebar_page = Some(SidebarLocation::Network);
    cena.files.sidebar_items = vec![SidebarItem {
        target: SidebarTarget::Remote {
            uri: uri.clone(),
            directory: true,
        },
        name: "Fotos".to_owned(),
    }];
    let task = cena.files.open_sidebar_item(0);
    assert_eq!(cena.files.network_pending.as_deref(), Some(uri.as_str()));
    complete(&mut cena, task);
    assert_eq!(cena.files.network_uri.as_deref(), Some(uri.as_str()));
    assert_eq!(cena.files.sidebar_items[0].name, "imagem.txt");
}

#[test]
fn resposta_atrasada_nao_reabre_pagina_de_rede() {
    let mut cena = Cena::nova();
    let uri = gio::File::for_path(cena.work()).uri().to_string();
    cena.files.sidebar_page = Some(SidebarLocation::Network);
    let task = cena.files.load_network(uri);
    cena.files.home();
    complete(&mut cena, task);
    assert_eq!(cena.files.sidebar_page, None);
    assert_eq!(cena.files.network_pending, None);
}
