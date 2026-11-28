use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::ChooserEvent;
use super::test_support::{abrir, caminhos, cena, filtro, indice, mandar, nomes};
use crate::app::message::Message;
use crate::domain::choice::Purpose;

#[test]
fn ativar_um_arquivo_devolve_o_caminho_sem_abrir_em_outro_aplicativo() {
    let (mut cena, respostas) = cena(abrir(), Vec::new());
    let indice = indice(&cena, "a.txt");
    mandar(&mut cena, Message::Activated(indice));
    assert_eq!(caminhos(&respostas), vec![vec![cena.work().join("a.txt")]]);
    assert!(cena.abertos().is_empty());
}

#[test]
fn o_filtro_esconde_arquivos_e_trocar_de_filtro_mostra_de_novo() {
    let filtros = vec![filtro("Vídeos", "*.mp4"), filtro("Todos", "*")];
    let (mut cena, _) = cena(abrir(), filtros);
    assert_eq!(nomes(&cena), ["Fotos", "demo.mp4"]);
    mandar(&mut cena, Message::Chooser(ChooserEvent::Filter(1)));
    assert_eq!(nomes(&cena), ["Fotos", "a.txt", "demo.mp4"]);
}

#[test]
fn salvar_usa_o_nome_e_confirma_antes_de_substituir() {
    let (mut cena, respostas) = cena(
        Purpose::Save {
            name: "novo.txt".to_owned(),
        },
        Vec::new(),
    );
    mandar(
        &mut cena,
        Message::Chooser(ChooserEvent::Name("a.txt".to_owned())),
    );
    mandar(&mut cena, Message::Chooser(ChooserEvent::Accept));
    assert!(caminhos(&respostas).is_empty());
    assert!(
        cena.files
            .chooser
            .as_ref()
            .is_some_and(|chooser| chooser.replace.is_some())
    );
    mandar(&mut cena, Message::Chooser(ChooserEvent::Replace));
    assert_eq!(caminhos(&respostas), vec![vec![cena.work().join("a.txt")]]);
}

#[test]
fn ao_salvar_clicar_duas_vezes_num_arquivo_copia_o_nome() {
    let (mut cena, respostas) = cena(
        Purpose::Save {
            name: "novo.txt".to_owned(),
        },
        Vec::new(),
    );
    let indice = indice(&cena, "demo.mp4");
    mandar(&mut cena, Message::Activated(indice));
    assert_eq!(
        cena.files
            .chooser
            .as_ref()
            .map(|chooser| chooser.name.as_str()),
        Some("demo.mp4")
    );
    assert!(caminhos(&respostas).is_empty());
}

#[test]
fn escolher_pasta_lista_so_pastas_e_sem_clique_devolve_a_atual() {
    let (mut cena, respostas) = cena(
        Purpose::Open {
            multiple: false,
            directory: true,
        },
        Vec::new(),
    );
    assert_eq!(nomes(&cena), ["Fotos"]);
    mandar(&mut cena, Message::Chooser(ChooserEvent::Accept));
    assert_eq!(caminhos(&respostas), vec![vec![cena.work()]]);
    mandar(&mut cena, Message::Clicked(0));
    mandar(&mut cena, Message::Chooser(ChooserEvent::Accept));
    assert_eq!(caminhos(&respostas)[1], vec![cena.work().join("Fotos")]);
}

#[test]
fn na_escolha_menus_e_teclas_que_mexem_em_arquivos_nao_funcionam() {
    let (mut cena, respostas) = cena(abrir(), Vec::new());
    let indice = indice(&cena, "a.txt");
    mandar(&mut cena, Message::Clicked(indice));
    mandar(&mut cena, Message::ContextEntry(indice));
    assert!(cena.files.context.is_none());
    mandar(&mut cena, Message::ContextBookmark(0));
    assert!(cena.files.context.is_none());
    mandar(
        &mut cena,
        Message::Key(Key::Named(Named::Delete), Modifiers::empty()),
    );
    mandar(
        &mut cena,
        Message::Key(Key::Character("x".into()), Modifiers::CTRL),
    );
    assert!(cena.work().join("a.txt").exists());
    assert!(caminhos(&respostas).is_empty());
}

#[test]
fn cancelar_ou_esc_nao_devolvem_nada() {
    let (mut cena, respostas) = cena(abrir(), Vec::new());
    mandar(&mut cena, Message::Escape);
    mandar(&mut cena, Message::Chooser(ChooserEvent::Cancel));
    assert!(caminhos(&respostas).is_empty());
}

#[test]
fn seletor_volta_e_avanca_pelo_historico_com_alt_e_setas() {
    let (mut cena, _) = cena(abrir(), Vec::new());
    let fotos = cena.work().join("Fotos");
    cena.files.go(fotos.clone());
    mandar(
        &mut cena,
        Message::Key(Key::Named(Named::ArrowLeft), Modifiers::ALT),
    );
    assert_eq!(cena.files.location, cena.work());
    mandar(
        &mut cena,
        Message::Key(Key::Named(Named::ArrowRight), Modifiers::ALT),
    );
    assert_eq!(cena.files.location, fotos);
}
