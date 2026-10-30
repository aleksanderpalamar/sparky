use std::fs;

use iced::keyboard::key::Named;
use sparky_ui::notice::Notice;

use super::message::{Message, TrashEvent};
use super::state::Mode;
use super::test_scene::Cena;
use crate::adapters::home_trash::HomeTrash;
use crate::ports::TrashStore;

#[test]
fn esvaziar_exige_confirmacao_e_inclui_itens_ocultos() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    let hidden = scene.work().join(".oculto.txt");
    fs::write(&hidden, "conteúdo").unwrap();
    trash.put(&hidden).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    assert!(scene.files.listing.is_empty());

    scene.mandar(Message::Trash(TrashEvent::Empty));
    assert_eq!(scene.files.mode, Mode::ConfirmEmptyTrash(1));
    scene.mandar(Message::Escape);
    assert_eq!(scene.files.mode, Mode::Browse);
    assert!(trash.directory().join(".oculto.txt").exists());

    scene.mandar(Message::Trash(TrashEvent::Empty));
    scene.nomeada(Named::Enter);
    assert!(scene.files.emptying);
    let result = trash.empty().map_err(|error| error.to_string());
    scene.mandar(Message::TrashEmptied(result));
    assert!(!scene.files.emptying);
    assert_eq!(scene.files.listing.total_len(), 0);
    assert!(matches!(scene.files.notice, Some(Notice::Info(_))));
}

#[test]
fn esvaziar_fora_da_lixeira_nao_altera_arquivos() {
    let mut scene = Cena::nova();
    scene.mandar(Message::Trash(TrashEvent::Empty));
    assert_eq!(scene.files.mode, Mode::Browse);
    assert!(scene.work().join("a.txt").exists());
}

#[test]
fn falha_ao_esvaziar_libera_o_botao_e_exibe_o_erro() {
    let mut scene = Cena::nova();
    let trash = HomeTrash::new(scene.dir.path().join("Trash"));
    trash.put(&scene.work().join("a.txt")).unwrap();
    scene.mandar(Message::Trash(TrashEvent::Open));
    scene.mandar(Message::Trash(TrashEvent::Empty));
    scene.nomeada(Named::Enter);
    scene.mandar(Message::TrashEmptied(Err("falha de leitura".to_owned())));

    assert!(!scene.files.emptying);
    assert_eq!(scene.files.listing.total_len(), 1);
    assert!(matches!(scene.files.notice, Some(Notice::Error(_))));
}
