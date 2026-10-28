use std::fs;

use iced::futures::StreamExt;
use iced_runtime::Action as TaskAction;

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use super::update::update;
use crate::domain::context::Action;

#[test]
fn copiar_e_mover_para_outra_pasta_nao_alteram_clipboard() {
    let mut scene = Cena::nova();
    let destination = scene.dir.path().join("destino");
    fs::create_dir(&destination).unwrap();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::CopyTo));
    scene.mandar(Message::Typed(destination.display().to_string()));
    scene.mandar(Message::Submit);
    assert!(destination.join("a.txt").exists());
    assert!(scene.work().join("a.txt").exists());
    assert!(scene.files.clip.is_none());
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::MoveTo));
    scene.mandar(Message::Typed(destination.display().to_string()));
    scene.mandar(Message::Submit);
    assert!(!scene.work().join("a.txt").exists());
    assert!(destination.join("a (cópia).txt").exists());
    assert!(scene.files.clip.is_none());
}

#[test]
fn destino_invalido_nao_apaga_a_origem() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::MoveTo));
    scene.mandar(Message::Typed(
        scene.dir.path().join("ausente").display().to_string(),
    ));
    scene.mandar(Message::Submit);
    assert!(scene.work().join("a.txt").exists());
    assert!(scene.files.notice.is_some());
}

#[test]
fn abrir_com_lista_aplicativos_e_abre_somente_uma_escolha_valida() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::OpenWith));
    assert_eq!(
        scene.files.open_with.as_ref().unwrap().apps[0].id,
        "teste.desktop"
    );
    scene.mandar(Message::OpenWithChoice("inexistente.desktop".to_owned()));
    assert!(scene.files.open_with.is_some());
    scene.mandar(Message::OpenWithChoice("teste.desktop".to_owned()));
    assert!(scene.files.open_with.is_none());
    assert_eq!(scene.abertos(), vec![scene.work().join("a.txt")]);
}

#[test]
fn comprimir_cria_arquivo_sem_bloquear_o_ciclo_de_eventos() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::Compress));
    assert!(matches!(scene.files.mode, Mode::Compress(_)));
    scene.mandar(Message::Typed("documento.tar.gz".to_owned()));
    let task = update(&mut scene.files, Message::Submit);
    assert!(scene.files.archive_work.is_some());
    let mut stream = iced_runtime::task::into_stream(task).unwrap();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async {
            while let Some(TaskAction::Output(message)) = stream.next().await {
                let _ = update(&mut scene.files, message);
            }
        });
    assert!(scene.work().join("documento.tar.gz").exists());
    assert!(scene.files.archive_work.is_none());
}

#[test]
fn enviar_arquivo_por_email_e_impedir_pasta() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::Email));
    assert_eq!(scene.abertos(), vec![scene.work().join("a.txt")]);
    scene.mandar(Message::ContextEntry(0));
    scene.mandar(Message::ContextAction(Action::Email));
    assert_eq!(scene.abertos().len(), 1);
}

#[test]
fn acao_desabilitada_ou_de_outro_menu_nao_e_executada() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextBackground);
    scene.mandar(Message::ContextAction(Action::Paste));
    assert!(scene.files.clip.is_none());
    scene.mandar(Message::ContextBackground);
    scene.mandar(Message::ContextAction(Action::Trash));
    assert!(scene.work().join("a.txt").exists());
    scene.mandar(Message::ContextEntry(0));
    scene.mandar(Message::ContextAction(Action::Email));
    assert!(scene.abertos().is_empty());
}
