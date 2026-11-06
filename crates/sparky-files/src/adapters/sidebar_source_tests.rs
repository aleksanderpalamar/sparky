use gio::prelude::*;
use std::fs;

use crate::adapters::sidebar_source::DesktopSidebar;
use crate::domain::sidebar_location::{SidebarLocation, SidebarTarget};
use crate::ports::SidebarSource;

#[test]
fn favoritos_persistem_e_remocao_atualiza_a_lista() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("arquivo.txt");
    fs::write(&path, "conteúdo").unwrap();
    let source = DesktopSidebar::new(dir.path().join("data"));
    source.favorite(&path, true).unwrap();
    let reopened = DesktopSidebar::new(dir.path().join("data"));
    assert!(reopened.is_favorite(&path));
    assert_eq!(
        reopened.list(SidebarLocation::Favorites).unwrap()[0].target,
        SidebarTarget::Local(path.clone())
    );
    reopened.favorite(&path, false).unwrap();
    assert!(
        reopened
            .list(SidebarLocation::Favorites)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn recentes_ignoram_arquivo_removido_e_ordenam_pela_visita() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    fs::create_dir(&data).unwrap();
    let first = dir.path().join("primeiro.txt");
    let last = dir.path().join("ultimo.txt");
    fs::write(&first, "a").unwrap();
    fs::write(&last, "b").unwrap();
    let uri = |path| gio::File::for_path(path).uri();
    let xbel = format!(
        "<xbel><bookmark href=\"{}\" visited=\"2026-01-01T00:00:00Z\"/><bookmark href=\"{}\" visited=\"2026-02-01T00:00:00Z\"/><bookmark href=\"file:///inexistente\"/></xbel>",
        uri(&first),
        uri(&last)
    );
    fs::write(data.join("recently-used.xbel"), xbel).unwrap();
    let items = DesktopSidebar::new(data)
        .list(SidebarLocation::Recent)
        .unwrap();
    assert_eq!(
        items.iter().map(|item| &item.target).collect::<Vec<_>>(),
        [&SidebarTarget::Local(last), &SidebarTarget::Local(first)]
    );
}

#[test]
fn renomear_diretorio_mantem_favoritos_filhos() {
    let dir = tempfile::tempdir().unwrap();
    let before = dir.path().join("antes");
    let after = dir.path().join("depois");
    let child = before.join("arquivo.txt");
    fs::create_dir(&before).unwrap();
    fs::write(&child, "dados").unwrap();
    let source = DesktopSidebar::new(dir.path().join("data"));
    source.favorite(&child, true).unwrap();
    fs::rename(&before, &after).unwrap();
    source.relocated(&before, &after).unwrap();
    assert!(source.is_favorite(&after.join("arquivo.txt")));
}

#[test]
fn rede_lista_arquivos_por_uri_gio() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("arquivo.txt"), "dados").unwrap();
    let source = DesktopSidebar::new(dir.path().join("data"));
    let uri = gio::File::for_path(dir.path()).uri();
    let items = source.network_children(&uri).unwrap();
    assert!(items.iter().any(|item| item.name == "arquivo.txt"
        && matches!(
            item.target,
            SidebarTarget::Remote {
                directory: false,
                ..
            }
        )));
}
