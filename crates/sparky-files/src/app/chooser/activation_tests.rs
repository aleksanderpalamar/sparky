use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::ChooserEvent;
use super::test_support::{abrir, caminhos, cena, indice, mandar};
use crate::app::message::Message;
use crate::domain::choice::Purpose;
use crate::domain::grid::ViewMode;

fn direita() -> Message {
    Message::Key(Key::Named(Named::ArrowRight), Modifiers::empty())
}

#[test]
fn na_lista_a_seta_direita_num_arquivo_devolve_o_caminho_sem_abrir_outro_aplicativo() {
    let (mut cena, respostas) = cena(abrir(), Vec::new());
    cena.files.view_mode = ViewMode::List;
    let indice = indice(&cena, "a.txt");
    mandar(&mut cena, Message::Clicked(indice));
    mandar(&mut cena, direita());
    assert_eq!(caminhos(&respostas), vec![vec![cena.work().join("a.txt")]]);
    assert!(cena.abertos().is_empty());
}

#[test]
fn na_lista_a_seta_direita_numa_pasta_continua_entrando_nela() {
    let (mut cena, respostas) = cena(abrir(), Vec::new());
    cena.files.view_mode = ViewMode::List;
    let indice = indice(&cena, "Fotos");
    mandar(&mut cena, Message::Clicked(indice));
    mandar(&mut cena, direita());
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
    assert!(caminhos(&respostas).is_empty());
}

#[test]
fn salvar_varios_sem_escolher_pasta_salva_na_pasta_aberta() {
    let (mut cena, respostas) = cena(
        Purpose::SaveFiles {
            names: vec!["saida.txt".to_owned()],
        },
        Vec::new(),
    );
    mandar(&mut cena, Message::Chooser(ChooserEvent::Accept));
    assert_eq!(
        caminhos(&respostas),
        vec![vec![cena.work().join("saida.txt")]]
    );
}

#[test]
fn salvar_varios_numa_pasta_clicada_salva_dentro_dela() {
    let (mut cena, respostas) = cena(
        Purpose::SaveFiles {
            names: vec!["saida.txt".to_owned()],
        },
        Vec::new(),
    );
    let indice = indice(&cena, "Fotos");
    mandar(&mut cena, Message::Clicked(indice));
    mandar(&mut cena, Message::Chooser(ChooserEvent::Accept));
    assert_eq!(
        caminhos(&respostas),
        vec![vec![cena.work().join("Fotos/saida.txt")]]
    );
}
