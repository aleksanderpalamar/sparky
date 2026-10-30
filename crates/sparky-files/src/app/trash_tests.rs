use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use iced::keyboard::key::Named;
use sparky_ui::notice::Notice;

use super::message::{Message, TrashEvent};
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::domain::context::{Action, Target};
use crate::domain::trash_info;
use crate::ports::TrashStore;

struct ChildFirstTrash(HomeTrash);

impl TrashStore for ChildFirstTrash {
    fn open(&self) -> io::Result<PathBuf> {
        self.0.open()
    }

    fn directory(&self) -> PathBuf {
        self.0.directory()
    }

    fn names(&self) -> io::Result<Vec<OsString>> {
        Ok(vec!["nota.txt".into(), "Fotos".into()])
    }

    fn restore(&self, name: &OsStr) -> io::Result<PathBuf> {
        self.0.restore(name)
    }

    fn empty(&self) -> io::Result<crate::ports::EmptyReport> {
        self.0.empty()
    }
}

#[test]
fn abre_lixeira_vazia_e_volta_para_a_pasta_pessoal() {
    let mut scene = Cena::nova();
    scene.mandar(Message::Trash(TrashEvent::Open));
    assert!(scene.files.in_trash());
    assert!(scene.files.listing.is_empty());
    scene.nomeada(Named::Backspace);
    assert_eq!(scene.files.location, scene.files.home);
}

#[test]
fn restaura_somente_o_item_escolhido_pelo_menu() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let first = scene.work().join("a.txt");
    let second = scene.work().join("b.txt");
    fs::write(&second, "outro").unwrap();
    trash.put(&first).unwrap();
    trash.put(&second).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.files.listing.select_name("a.txt");
    scene.mandar(Message::ContextEntry(scene.files.listing.cursor()));
    assert_eq!(
        scene.files.context.as_ref().map(|menu| menu.target),
        Some(Target::Trashed)
    );
    scene.mandar(Message::ContextAction(Action::Restore));
    assert!(first.exists());
    assert!(!second.exists());
    assert_eq!(scene.files.listing.len(), 1);
}

#[test]
fn restaurar_todos_preserva_conflitos_e_restaura_os_demais() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let first = scene.work().join("a.txt");
    let second = scene.work().join("b.txt");
    fs::write(&second, "antigo").unwrap();
    trash.put(&first).unwrap();
    trash.put(&second).unwrap();
    fs::write(&first, "novo").unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::RestoreAll));
    assert_eq!(fs::read_to_string(&first).unwrap(), "novo");
    assert_eq!(fs::read_to_string(&second).unwrap(), "antigo");
    assert_eq!(scene.files.listing.len(), 1);
    assert!(matches!(scene.files.notice, Some(Notice::Error(_))));
}

#[test]
fn na_lixeira_teclas_de_apagar_e_criar_nao_alteram_itens() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    trash.put(&scene.work().join("a.txt")).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.nomeada(Named::Delete);
    scene.tecla("a");
    assert!(scene.files.in_trash());
    assert!(scene.dir.path().join("Trash/files/a.txt").exists());
    assert!(matches!(scene.files.mode, super::state::Mode::Browse));
}

#[test]
fn restaurar_todos_inclui_itens_ocultos_pela_visualizacao() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let hidden = scene.work().join(".oculto.txt");
    fs::write(&hidden, "segredo").unwrap();
    trash.put(&hidden).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    assert!(scene.files.listing.is_empty());
    assert_eq!(scene.files.listing.total_len(), 1);
    scene.mandar(Message::Trash(TrashEvent::RestoreAll));
    assert_eq!(fs::read_to_string(hidden).unwrap(), "segredo");
}

#[test]
fn restaurar_todos_recupera_pasta_antes_do_arquivo_filho() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let folder = scene.work().join("Fotos");
    let child = folder.join("nota.txt");
    fs::write(&child, "conteúdo").unwrap();
    trash.put(&child).unwrap();
    trash.put(&folder).unwrap();
    scene.files.services.trash = std::sync::Arc::new(ChildFirstTrash(trash));
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::RestoreAll));
    assert_eq!(fs::read_to_string(child).unwrap(), "conteúdo");
    assert_eq!(scene.files.listing.len(), 0);
}

#[test]
fn restaurar_todos_encerra_quando_o_pai_nao_pode_ser_recuperado() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let folder = scene.work().join("Fotos");
    let child = folder.join("nota.txt");
    fs::write(&child, "conteúdo").unwrap();
    trash.put(&child).unwrap();
    fs::remove_dir(&folder).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::RestoreAll));
    assert!(scene.dir.path().join("Trash/files/nota.txt").exists());
    assert!(matches!(scene.files.notice, Some(Notice::Error(_))));
}

#[test]
fn restaurar_todos_preserva_nome_nao_utf8_de_item_externo() {
    let mut scene = Cena::nova();
    let original = external_item(&scene);
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::RestoreAll));
    assert_eq!(fs::read_to_string(original).unwrap(), "conteúdo");
}

#[test]
fn restaurar_selecionado_preserva_nome_nao_utf8_de_item_externo() {
    let mut scene = Cena::nova();
    let original = external_item(&scene);
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::RestoreSelected));
    assert_eq!(fs::read_to_string(original).unwrap(), "conteúdo");
}

fn external_item(scene: &Cena) -> PathBuf {
    let root = scene.dir.path().join("Trash");
    let original = scene.work().join("restaurado.txt");
    fs::create_dir_all(root.join("files")).unwrap();
    fs::create_dir_all(root.join("info")).unwrap();
    let name = std::ffi::OsString::from_vec(b"imagem\xff.png".to_vec());
    fs::write(root.join("files").join(&name), "conteúdo").unwrap();
    let mut record = name.clone();
    record.push(".trashinfo");
    fs::write(
        root.join("info").join(record),
        trash_info::info(&original, &chrono::Local::now().naive_local()),
    )
    .unwrap();
    original
}
