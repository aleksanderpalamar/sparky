use std::fs;

use iced::keyboard::{Key, Modifiers};

use super::jobs::JobEvent;
use super::message::Message;
use super::test_scene::Cena;
use crate::domain::clip::{Clip, Pasted};
use crate::domain::context::Action;
use crate::domain::transfer::{Ending, Outcome};

fn ctrl(cena: &mut Cena, letra: &str) {
    cena.concluir(Message::Key(Key::Character(letra.into()), Modifiers::CTRL));
}

fn aviso(cena: &Cena) -> Option<String> {
    cena.files
        .notice
        .as_ref()
        .map(|notice| notice.text().to_owned())
}

#[test]
fn copiar_e_recortar_publicam_a_lista_de_arquivos() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    ctrl(&mut cena, "c");
    ctrl(&mut cena, "x");
    let ofertas = cena.area.ofertas.borrow();
    let a = cena.work().join("a.txt");
    assert_eq!(ofertas[0].clip, Clip::Copy(vec![a.clone()]));
    assert_eq!(ofertas[1].clip, Clip::Cut(vec![a]));
    assert!(ofertas.iter().all(|offer| offer.image.is_none()));
}

#[test]
fn colar_traz_arquivos_copiados_em_outro_aplicativo() {
    let mut cena = Cena::nova();
    let externo = cena.dir.path().join("relatorio.pdf");
    fs::write(&externo, "pdf").unwrap();
    *cena.area.leitura.borrow_mut() = Pasted::Files(Clip::Copy(vec![externo.clone()]));
    ctrl(&mut cena, "v");
    assert_eq!(
        fs::read_to_string(cena.work().join("relatorio.pdf")).unwrap(),
        "pdf"
    );
    assert!(externo.exists());
}

#[test]
fn recorte_vindo_de_outro_aplicativo_move_os_arquivos() {
    let mut cena = Cena::nova();
    let externo = cena.dir.path().join("musica.mp3");
    fs::write(&externo, "mp3").unwrap();
    *cena.area.leitura.borrow_mut() = Pasted::Files(Clip::Cut(vec![externo.clone()]));
    ctrl(&mut cena, "v");
    assert!(cena.work().join("musica.mp3").exists());
    assert!(!externo.exists());
    assert_eq!(*cena.area.limpezas.borrow(), 1);
}

#[test]
fn texto_na_area_de_transferencia_nao_cola_arquivos_antigos() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    ctrl(&mut cena, "c");
    *cena.area.leitura.borrow_mut() = Pasted::Other;
    ctrl(&mut cena, "v");
    assert!(!cena.work().join("a (cópia).txt").exists());
    assert_eq!(aviso(&cena).as_deref(), Some("Nada para colar"));
    assert!(cena.files.clip.is_none());
}

#[test]
fn sem_area_do_sistema_cola_a_copia_interna() {
    let mut cena = Cena::nova();
    cena.files.listing.select_name("a.txt");
    ctrl(&mut cena, "c");
    ctrl(&mut cena, "v");
    assert!(cena.work().join("a (cópia).txt").exists());
}

#[test]
fn mover_para_outra_pasta_nao_limpa_a_area_de_transferencia() {
    let mut cena = Cena::nova();
    let destino = cena.dir.path().join("destino");
    fs::create_dir(&destino).unwrap();
    cena.files.listing.select_name("a.txt");
    ctrl(&mut cena, "c");
    let indice = cena
        .files
        .listing
        .visible()
        .position(|entry| entry.name == "a.txt")
        .unwrap_or(usize::MAX);
    cena.mandar(Message::ContextEntry(indice));
    cena.mandar(Message::ContextAction(Action::MoveTo));
    cena.mandar(Message::Typed(destino.display().to_string()));
    cena.concluir(Message::Submit);
    assert!(destino.join("a.txt").exists());
    assert_eq!(*cena.area.limpezas.borrow(), 0);
    assert!(cena.files.clip.is_some());
}

#[test]
fn recorte_concluido_nao_limpa_uma_selecao_mais_nova() {
    let mut cena = Cena::nova();
    let externo = cena.dir.path().join("musica.mp3");
    fs::write(&externo, "mp3").unwrap();
    *cena.area.leitura.borrow_mut() = Pasted::Files(Clip::Cut(vec![externo.clone()]));
    cena.mandar(Message::Key(Key::Character("v".into()), Modifiers::CTRL));
    let nova = Clip::Copy(vec![cena.work().join("a.txt")]);
    *cena.area.leitura.borrow_mut() = Pasted::Files(nova.clone());
    cena.files.job_event(JobEvent::Done(Outcome {
        completed: Vec::new(),
        displaced: Vec::new(),
        skipped: 0,
        ending: Ending::Finished,
    }));
    assert_eq!(*cena.area.limpezas.borrow(), 0);
    assert_eq!(cena.files.clip, Some(nova));
}
