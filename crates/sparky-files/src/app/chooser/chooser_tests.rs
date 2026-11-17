use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::{Chooser, ChooserEvent, update};
use crate::app::message::Message;
use crate::app::test_scene::Cena;
use crate::domain::choice::{Filter, Outcome, Purpose, Request, Rule};
use crate::ports::chooser::Answer;

#[derive(Default)]
struct Respostas(RefCell<Vec<Outcome>>);

impl Answer for Respostas {
    fn send(&self, outcome: &Outcome) {
        self.0.borrow_mut().push(outcome.clone());
    }
}

fn sem_mime(_: &str) -> String {
    "application/octet-stream".to_owned()
}

fn filtro(nome: &str, padrao: &str) -> Filter {
    Filter {
        name: nome.to_owned(),
        rules: vec![Rule::Glob(padrao.to_owned())],
    }
}

fn cena(purpose: Purpose, filters: Vec<Filter>) -> (Cena, Rc<Respostas>) {
    let mut cena = Cena::nova();
    assert!(fs::write(cena.work().join("demo.mp4"), "video").is_ok());
    let respostas = Rc::new(Respostas::default());
    let pedido = Request {
        purpose,
        title: String::new(),
        accept: None,
        folder: None,
        filters,
        filter: None,
    };
    cena.files.chooser = Some(Chooser::new(pedido, respostas.clone(), sem_mime));
    cena.files.refresh();
    (cena, respostas)
}

fn mandar(cena: &mut Cena, message: Message) {
    let _ = update(&mut cena.files, message);
}

fn indice(cena: &Cena, nome: &str) -> usize {
    (0..cena.files.listing.len())
        .find(|index| {
            cena.files
                .listing
                .get(*index)
                .is_some_and(|entry| entry.name == nome)
        })
        .unwrap_or_else(|| panic!("{nome} não está na lista"))
}

fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|entry| entry.name.clone())
        .collect()
}

fn caminhos(respostas: &Respostas) -> Vec<Vec<PathBuf>> {
    respostas
        .0
        .borrow()
        .iter()
        .map(|outcome| outcome.paths.clone())
        .collect()
}

fn abrir() -> Purpose {
    Purpose::Open {
        multiple: false,
        directory: false,
    }
}

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
