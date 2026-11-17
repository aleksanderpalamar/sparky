use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use super::{Chooser, update};
use crate::app::message::Message;
use crate::app::test_scene::Cena;
use crate::domain::choice::{Filter, Outcome, Purpose, Request, Rule};
use crate::ports::chooser::Answer;

#[derive(Default)]
pub struct Respostas(pub RefCell<Vec<Outcome>>);

impl Answer for Respostas {
    fn send(&self, outcome: &Outcome) {
        self.0.borrow_mut().push(outcome.clone());
    }
}

pub fn sem_mime(_: &str) -> String {
    "application/octet-stream".to_owned()
}

pub fn filtro(nome: &str, padrao: &str) -> Filter {
    Filter {
        name: nome.to_owned(),
        rules: vec![Rule::Glob(padrao.to_owned())],
    }
}

pub fn cena(purpose: Purpose, filters: Vec<Filter>) -> (Cena, Rc<Respostas>) {
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

pub fn mandar(cena: &mut Cena, message: Message) {
    let _ = update(&mut cena.files, message);
}

pub fn indice(cena: &Cena, nome: &str) -> usize {
    (0..cena.files.listing.len())
        .find(|index| {
            cena.files
                .listing
                .get(*index)
                .is_some_and(|entry| entry.name == nome)
        })
        .unwrap_or_else(|| panic!("{nome} não está na lista"))
}

pub fn nomes(cena: &Cena) -> Vec<String> {
    cena.files
        .listing
        .visible()
        .map(|entry| entry.name.clone())
        .collect()
}

pub fn caminhos(respostas: &Respostas) -> Vec<Vec<PathBuf>> {
    respostas
        .0
        .borrow()
        .iter()
        .map(|outcome| outcome.paths.clone())
        .collect()
}

pub fn abrir() -> Purpose {
    Purpose::Open {
        multiple: false,
        directory: false,
    }
}
