use std::fs;

use iced::Point;
use iced::keyboard::Modifiers;

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use crate::domain::drop::DropTarget;
use crate::domain::icons::IconKind;
use crate::domain::places::Place;
use crate::domain::removal::Removal;

fn indice(cena: &Cena, nome: &str) -> usize {
    cena.files
        .listing
        .visible()
        .position(|entry| entry.name == nome)
        .unwrap_or(usize::MAX)
}

fn arrastar(cena: &mut Cena, nome: &str) {
    cena.mandar(Message::Pointer(Point::new(10.0, 10.0)));
    let posicao = indice(cena, nome);
    cena.mandar(Message::Clicked(posicao));
    cena.mandar(Message::Pointer(Point::new(40.0, 10.0)));
}

fn soltar_em(cena: &mut Cena, alvo: DropTarget) {
    cena.mandar(Message::Hover(alvo));
    cena.concluir(Message::Released);
}

#[test]
fn soltar_um_arquivo_sobre_uma_pasta_move_e_pode_desfazer() {
    let mut cena = Cena::nova();
    arrastar(&mut cena, "a.txt");
    let fotos = indice(&cena, "Fotos");
    soltar_em(&mut cena, DropTarget::Entry(fotos));
    assert!(cena.work().join("Fotos/a.txt").is_file());
    assert!(!cena.work().join("a.txt").exists());
    assert_eq!(cena.files.journal.len(), 1);
    assert_eq!(*cena.area.limpezas.borrow(), 0);
}

#[test]
fn com_ctrl_pressionado_soltar_copia() {
    let mut cena = Cena::nova();
    cena.mandar(Message::Modifiers(Modifiers::CTRL));
    arrastar(&mut cena, "a.txt");
    let fotos = indice(&cena, "Fotos");
    soltar_em(&mut cena, DropTarget::Entry(fotos));
    assert!(cena.work().join("Fotos/a.txt").is_file());
    assert!(cena.work().join("a.txt").is_file());
}

#[test]
fn soltar_fora_de_um_alvo_ou_cancelar_com_esc_nao_faz_nada() {
    let mut cena = Cena::nova();
    arrastar(&mut cena, "a.txt");
    cena.concluir(Message::Released);
    arrastar(&mut cena, "a.txt");
    let fotos = indice(&cena, "Fotos");
    cena.mandar(Message::Hover(DropTarget::Entry(fotos)));
    cena.mandar(Message::Escape);
    cena.concluir(Message::Released);
    assert!(cena.work().join("a.txt").is_file());
    assert!(!cena.work().join("Fotos/a.txt").exists());
}

#[test]
fn movimento_pequeno_e_sair_do_alvo_nao_soltam_nada() {
    let mut cena = Cena::nova();
    cena.mandar(Message::Pointer(Point::new(10.0, 10.0)));
    let posicao = indice(&cena, "a.txt");
    cena.mandar(Message::Clicked(posicao));
    cena.mandar(Message::Pointer(Point::new(13.0, 10.0)));
    let fotos = indice(&cena, "Fotos");
    soltar_em(&mut cena, DropTarget::Entry(fotos));
    assert!(cena.work().join("a.txt").is_file());
    arrastar(&mut cena, "a.txt");
    cena.mandar(Message::Hover(DropTarget::Entry(fotos)));
    cena.mandar(Message::Unhover(DropTarget::Entry(fotos)));
    cena.concluir(Message::Released);
    assert!(cena.work().join("a.txt").is_file());
}

#[test]
fn uma_pasta_nao_pode_ser_solta_sobre_ela_mesma_nem_sobre_arquivo() {
    let mut cena = Cena::nova();
    arrastar(&mut cena, "Fotos");
    let fotos = indice(&cena, "Fotos");
    soltar_em(&mut cena, DropTarget::Entry(fotos));
    arrastar(&mut cena, "Fotos");
    let arquivo = indice(&cena, "a.txt");
    soltar_em(&mut cena, DropTarget::Entry(arquivo));
    assert!(cena.work().join("Fotos").is_dir());
    assert!(cena.files.job.is_none());
}

#[test]
fn soltar_num_lugar_da_lateral_ou_numa_aba_leva_para_a_pasta_dele() {
    let mut cena = Cena::nova();
    let destino = cena.dir.path().join("Documentos");
    assert!(fs::create_dir(&destino).is_ok());
    cena.files.places = vec![Place {
        key: "place-documents",
        icon: IconKind::Folder,
        path: destino.clone(),
    }];
    arrastar(&mut cena, "a.txt");
    soltar_em(&mut cena, DropTarget::Place(0));
    assert!(destino.join("a.txt").is_file());
    cena.mandar(Message::Key(
        iced::keyboard::Key::Character("t".into()),
        Modifiers::CTRL,
    ));
    cena.files.go(destino.clone());
    cena.mandar(Message::Released);
    arrastar(&mut cena, "a.txt");
    soltar_em(&mut cena, DropTarget::Tab(0));
    assert!(cena.work().join("a.txt").is_file());
}

#[test]
fn soltar_na_lixeira_pede_confirmacao() {
    let mut cena = Cena::nova();
    arrastar(&mut cena, "a.txt");
    soltar_em(&mut cena, DropTarget::Trash);
    assert_eq!(
        cena.files.mode,
        Mode::ConfirmRemoval(Removal::Trash, vec!["a.txt".to_owned()])
    );
}
