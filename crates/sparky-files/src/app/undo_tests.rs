use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;
use crate::domain::context::Action;

fn desfazer(cena: &mut Cena) {
    cena.concluir(Message::Key(Key::Character("z".into()), Modifiers::CTRL));
}

fn aviso(cena: &Cena) -> Option<String> {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
}

#[test]
fn desfazer_renomear_volta_ao_nome_antigo() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("r");
    cena.mandar(Message::Typed("b.txt".to_owned()));
    cena.mandar(Message::Submit);
    desfazer(&mut cena);
    assert!(cena.work().join("a.txt").exists());
    assert!(!cena.work().join("b.txt").exists());
    assert_eq!(aviso(&cena).as_deref(), Some("Ação desfeita"));
}

#[test]
fn desfazer_lixeira_restaura_o_item() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("d");
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("a.txt").exists());
    desfazer(&mut cena);
    assert_eq!(fs::read_to_string(cena.work().join("a.txt")).unwrap(), "oi");
}

#[test]
fn desfazer_criacao_manda_a_pasta_nova_para_a_lixeira() {
    let mut cena = Cena::nova();
    cena.tecla("a");
    cena.mandar(Message::Typed("Projetos/".to_owned()));
    cena.mandar(Message::Submit);
    desfazer(&mut cena);
    assert!(!cena.work().join("Projetos").exists());
    assert!(cena.dir.path().join("Trash/files/Projetos").is_dir());
}

#[test]
fn desfazer_mover_traz_o_item_de_volta_em_segundo_plano() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("x");
    assert!(cena.files.go(cena.work().join("Fotos")));
    cena.tecla_concluida("p");
    assert!(cena.work().join("Fotos/a.txt").exists());
    desfazer(&mut cena);
    assert!(cena.work().join("a.txt").exists());
    assert!(!cena.work().join("Fotos/a.txt").exists());
    assert!(cena.files.job.is_none());
}

#[test]
fn desfazer_copia_manda_a_copia_para_a_lixeira() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("y");
    cena.tecla_concluida("p");
    assert!(cena.work().join("a (cópia).txt").exists());
    desfazer(&mut cena);
    assert!(!cena.work().join("a (cópia).txt").exists());
    assert!(cena.work().join("a.txt").exists());
}

#[test]
fn sem_historico_avisa_que_nao_ha_o_que_desfazer() {
    let mut cena = Cena::nova();
    desfazer(&mut cena);
    assert_eq!(aviso(&cena).as_deref(), Some("Nada para desfazer"));
}

fn substituir_em_destino(cena: &mut Cena, acao: Action) -> std::path::PathBuf {
    let destino = cena.dir.path().join("destino");
    assert!(fs::create_dir(&destino).is_ok());
    assert!(fs::write(destino.join("a.txt"), "velho").is_ok());
    let indice = cena
        .files
        .listing
        .visible()
        .position(|entry| entry.name == "a.txt")
        .unwrap_or(usize::MAX);
    cena.mandar(Message::ContextEntry(indice));
    cena.mandar(Message::ContextAction(acao));
    cena.mandar(Message::Typed(destino.display().to_string()));
    cena.mandar(Message::Submit);
    cena.concluir(Message::Key(Key::Character("s".into()), Modifiers::empty()));
    assert_eq!(
        fs::read_to_string(destino.join("a.txt")).ok().as_deref(),
        Some("oi")
    );
    destino
}

#[test]
fn desfazer_copia_que_substituiu_devolve_o_item_antigo() {
    let mut cena = Cena::nova();
    let destino = substituir_em_destino(&mut cena, Action::CopyTo);
    desfazer(&mut cena);
    assert_eq!(
        fs::read_to_string(destino.join("a.txt")).ok().as_deref(),
        Some("velho")
    );
    assert_eq!(
        fs::read_to_string(cena.work().join("a.txt"))
            .ok()
            .as_deref(),
        Some("oi")
    );
}

#[test]
fn desfazer_movimento_que_substituiu_devolve_os_dois_itens() {
    let mut cena = Cena::nova();
    let destino = substituir_em_destino(&mut cena, Action::MoveTo);
    desfazer(&mut cena);
    assert_eq!(
        fs::read_to_string(destino.join("a.txt")).ok().as_deref(),
        Some("velho")
    );
    assert_eq!(
        fs::read_to_string(cena.work().join("a.txt"))
            .ok()
            .as_deref(),
        Some("oi")
    );
}
