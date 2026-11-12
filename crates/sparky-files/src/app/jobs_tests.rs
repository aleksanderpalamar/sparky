use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::keyboard::key::Named;

use super::jobs::{Job, Origin};
use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use super::test_tasks::concluir;
use crate::domain::clip::Clip;
use crate::domain::transfer::{Action, Progress};
use crate::view::status::progress;

fn com_conflito() -> (Cena, PathBuf) {
    let mut cena = Cena::nova();
    let fotos = cena.work().join("Fotos");
    fs::write(fotos.join("a.txt"), "antiga").unwrap();
    let clip = Clip::Copy(vec![cena.work().join("a.txt")]);
    let task = cena.files.transfer(clip, fotos.clone());
    concluir(&mut cena.files, task, 100);
    (cena, fotos)
}

#[test]
fn colar_comeca_em_segundo_plano_e_recusa_outra_operacao() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("y");
    cena.tecla("p");
    assert!(cena.files.job.is_some());
    cena.tecla("p");
    let aviso = cena
        .files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned());
    assert_eq!(
        aviso.as_deref(),
        Some("Aguarde a operação em andamento terminar")
    );
}

#[test]
fn esc_pede_o_cancelamento_da_operacao() {
    let mut cena = Cena::nova();
    let cancel = Arc::new(AtomicBool::new(false));
    cena.files.job = Some(Job {
        cancel: cancel.clone(),
        progress: Progress::default(),
        action: Action::Copy,
        origin: Origin::User,
    });
    cena.nomeada(Named::Escape);
    cena.mandar(Message::Escape);
    assert!(cancel.load(Ordering::SeqCst));
}

#[test]
fn nome_existente_pergunta_antes_de_copiar() {
    let (cena, fotos) = com_conflito();
    assert!(matches!(&cena.files.mode, Mode::Conflict(pending) if pending.names == ["a.txt"]));
    assert_eq!(fs::read_to_string(fotos.join("a.txt")).unwrap(), "antiga");
}

#[test]
fn pular_mantem_o_item_do_destino() {
    let (mut cena, fotos) = com_conflito();
    cena.tecla_concluida("p");
    assert_eq!(fs::read_to_string(fotos.join("a.txt")).unwrap(), "antiga");
    let aviso = cena
        .files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned());
    assert_eq!(aviso.as_deref(), Some("Nenhum item copiado · 1 pulado"));
}

#[test]
fn manter_os_dois_cria_uma_copia_com_outro_nome() {
    let (mut cena, fotos) = com_conflito();
    cena.tecla_concluida("m");
    assert_eq!(fs::read_to_string(fotos.join("a.txt")).unwrap(), "antiga");
    assert_eq!(
        fs::read_to_string(fotos.join("a (cópia).txt")).unwrap(),
        "oi"
    );
}

#[test]
fn substituir_troca_o_item_e_guarda_o_antigo_na_lixeira() {
    let (mut cena, fotos) = com_conflito();
    cena.tecla_concluida("s");
    assert_eq!(fs::read_to_string(fotos.join("a.txt")).unwrap(), "oi");
    let lixeira = cena.dir.path().join("Trash/files/a.txt");
    assert_eq!(fs::read_to_string(lixeira).unwrap(), "antiga");
}

#[test]
fn barra_de_status_mostra_o_andamento() {
    let job = Job {
        cancel: Arc::new(AtomicBool::new(false)),
        progress: Progress {
            items_done: 1,
            items_total: 2,
            bytes_done: 30,
            bytes_total: 60,
        },
        action: Action::Move,
        origin: Origin::User,
    };
    assert_eq!(progress(&job), "Movendo 1 de 2 · 50% · Esc cancela");
}
