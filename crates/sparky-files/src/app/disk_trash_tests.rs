use std::fs;
use std::sync::Arc;

use super::message::{Message, TrashEvent};
use super::test_scene::Cena;
use crate::adapters::disk_trash::prepare;
use crate::adapters::home_trash::HomeTrash;

#[test]
fn lixeira_de_outro_disco_abre_e_restaura_no_lugar_original() {
    let mut cena = Cena::nova();
    let disco = cena.dir.path().join("USB");
    fs::create_dir_all(disco.join("Fotos")).unwrap();
    fs::write(disco.join("Fotos/praia.png"), "png").unwrap();
    let raiz = prepare(&disco, rustix::process::getuid().as_raw()).unwrap();
    let lixeira = HomeTrash::on_disk(raiz, disco.clone());
    lixeira.put(&disco.join("Fotos/praia.png")).unwrap();
    cena.files.other_trashes = vec![Arc::new(lixeira)];
    cena.mandar(Message::Trash(TrashEvent::OpenOther(0)));
    assert!(cena.files.in_trash());
    assert_eq!(cena.selecionado().as_deref(), Some("praia.png"));
    cena.mandar(Message::Trash(TrashEvent::RestoreSelected));
    assert!(disco.join("Fotos/praia.png").exists());
    assert!(cena.files.listing.is_empty());
}

#[test]
fn lixeira_pessoal_continua_separada_das_outras() {
    let mut cena = Cena::nova();
    cena.mandar(Message::Trash(TrashEvent::Open));
    assert!(cena.files.in_trash());
    assert!(cena.files.other_trashes.is_empty());
    cena.mandar(Message::Trash(TrashEvent::OpenOther(3)));
    assert_eq!(cena.files.location, cena.dir.path().join("Trash/files"));
}
