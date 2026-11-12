use std::fs;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::test_scene::Cena;

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
