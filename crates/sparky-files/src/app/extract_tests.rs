use std::fs;

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::context::{Action, Target};

fn com_zip() -> Cena {
    let cena = Cena::nova();
    assert!(fs::write(cena.work().join("fotos.zip"), "zip").is_ok());
    let mut cena = cena;
    cena.mandar(Message::FolderChanged);
    cena.files.listing.select_name("fotos.zip");
    cena
}

fn aviso(cena: &Cena) -> String {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
        .unwrap_or_default()
}

#[test]
fn e_extrai_aqui_seleciona_o_resultado_e_ctrl_z_desfaz() {
    let mut cena = com_zip();
    cena.tecla_concluida("e");
    assert!(cena.work().join("fotos/conteudo.txt").is_file());
    assert!(cena.files.job.is_none());
    assert_eq!(cena.selecionado().as_deref(), Some("fotos"));
    assert_eq!(aviso(&cena), "Extraído em fotos");
    assert_eq!(cena.files.journal.len(), 1);
}

#[test]
fn desfazer_a_extracao_manda_a_pasta_para_a_lixeira() {
    let mut cena = com_zip();
    cena.tecla_concluida("e");
    cena.mandar(Message::Key(
        iced::keyboard::Key::Character("z".into()),
        iced::keyboard::Modifiers::CTRL,
    ));
    assert!(!cena.work().join("fotos").exists());
    assert!(cena.dir.path().join("Trash/files/fotos").exists());
}

#[test]
fn e_sobre_arquivo_que_nao_e_compactado_nao_faz_nada() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    cena.tecla_concluida("e");
    assert!(cena.files.job.is_none());
    assert!(!cena.work().join("a").exists());
}

#[test]
fn menu_de_um_compactado_oferece_extrair_aqui() {
    let mut cena = com_zip();
    let indice = cena.files.listing.cursor();
    cena.mandar(Message::ContextEntry(indice));
    assert_eq!(
        cena.files.context.as_ref().map(|menu| menu.target),
        Some(Target::Archive)
    );
    cena.concluir(Message::ContextAction(Action::Extract));
    assert!(cena.work().join("fotos/conteudo.txt").is_file());
}
