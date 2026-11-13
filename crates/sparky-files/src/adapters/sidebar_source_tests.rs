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
    source
        .relocated(&[(before.clone(), after.clone())])
        .unwrap();
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

#[test]
fn atalho_de_rede_abre_a_uri_de_destino() {
    let root = gio::File::for_uri("network:///");
    let info = gio::FileInfo::new();
    info.set_name("servidor");
    info.set_display_name("Servidor");
    info.set_file_type(gio::FileType::Shortcut);
    info.set_attribute_string("standard::target-uri", "smb://servidor/");
    let item = super::sidebar_network::item(&root, &info);
    assert_eq!(
        item.target,
        SidebarTarget::Remote {
            uri: "smb://servidor/".into(),
            directory: true
        }
    );
}

#[test]
fn rede_usa_uri_do_filho_quando_destino_falta_ou_entrada_e_comum() {
    let root = gio::File::for_uri("network:///");
    let info = gio::FileInfo::new();
    info.set_name("pasta");
    info.set_display_name("Pasta");
    info.set_file_type(gio::FileType::Shortcut);
    for target in [None, Some("")] {
        if let Some(target) = target {
            info.set_attribute_string("standard::target-uri", target);
        }
        assert_eq!(
            super::sidebar_network::item(&root, &info).target,
            SidebarTarget::Remote {
                uri: "network:///pasta".into(),
                directory: true
            }
        );
    }
    info.set_file_type(gio::FileType::Regular);
    info.set_attribute_string("standard::target-uri", "smb://outro/");
    assert_eq!(
        super::sidebar_network::item(&root, &info).target,
        SidebarTarget::Remote {
            uri: "network:///pasta".into(),
            directory: false
        }
    );
}

#[test]
fn favoritos_indisponiveis_sobrevivem_a_alteracoes_de_outros_itens() {
    let dir = tempfile::tempdir().unwrap();
    let drive = dir.path().join("unidade");
    let offline = dir.path().join("desconectada");
    fs::create_dir(&drive).unwrap();
    let saved = drive.join("arquivo.txt");
    let other = dir.path().join("outro.txt");
    fs::write(&saved, "dados").unwrap();
    fs::write(&other, "dados").unwrap();
    let data = dir.path().join("data");
    let source = DesktopSidebar::new(data.clone());
    source.favorite(&saved, true).unwrap();
    fs::rename(&drive, &offline).unwrap();
    source.favorite(&other, true).unwrap();
    source.favorite(&other, false).unwrap();
    let reopened = DesktopSidebar::new(data);
    assert!(reopened.is_favorite(&saved));
    assert!(
        reopened
            .list(SidebarLocation::Favorites)
            .unwrap()
            .is_empty()
    );
    fs::rename(&offline, &drive).unwrap();
    assert_eq!(
        reopened.list(SidebarLocation::Favorites).unwrap()[0].target,
        SidebarTarget::Local(saved.clone())
    );
    fs::rename(&drive, &offline).unwrap();
    reopened.favorite(&saved, false).unwrap();
    assert!(!reopened.is_favorite(&saved));
}
