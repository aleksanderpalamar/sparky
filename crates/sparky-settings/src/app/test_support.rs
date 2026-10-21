use std::cell::{Cell, RefCell};
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use sparky_ipc::OutputInfo;
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::{Context, Services, Settings};
use crate::app::update::update;
use crate::domain::about::About;
use crate::domain::keymaps::Keymaps;
use crate::ports::commands::{CommandError, CommandRunner};
use crate::ports::fake::FakeCommands;
use crate::ports::{Compositor, CompositorError, Store};

#[derive(Clone, Default)]
pub struct Comandos(pub Rc<FakeCommands>);

impl CommandRunner for Comandos {
    fn run(&self, program: &str, arguments: &[&str]) -> Result<String, CommandError> {
        self.0.run(program, arguments)
    }
}

#[derive(Clone, Default)]
pub struct Arquivo(pub Rc<RefCell<Option<String>>>);

impl Store for Arquivo {
    fn load(&self) -> io::Result<Option<String>> {
        Ok(self.0.borrow().clone())
    }

    fn save(&self, contents: &str) -> io::Result<()> {
        *self.0.borrow_mut() = Some(contents.to_owned());
        Ok(())
    }

    fn location(&self) -> PathBuf {
        PathBuf::from("/h/.config/sparky/sparky.toml")
    }
}

#[derive(Clone, Default)]
pub struct Sparky {
    pub recargas: Rc<Cell<usize>>,
    pub fora_do_ar: bool,
}

impl Compositor for Sparky {
    fn reload(&self) -> Result<(), CompositorError> {
        self.recargas.set(self.recargas.get() + 1);
        match self.fora_do_ar {
            true => Err(CompositorError::Unreachable("sem socket".to_owned())),
            false => Ok(()),
        }
    }

    fn outputs(&self) -> Result<Vec<OutputInfo>, CompositorError> {
        Ok(Vec::new())
    }
}

pub const REGRAS: &str = "! layout\n  us  English (US)\n  br  Portuguese (Brazil)\n\n! variant\n  abnt2  br: Portuguese (Brazil)\n";

pub fn painel(conteudo: Option<&str>, sparky: Sparky) -> (Settings, Arquivo) {
    painel_com(conteudo, sparky, Comandos::default())
}

pub fn painel_com(conteudo: Option<&str>, sparky: Sparky, comandos: Comandos) -> (Settings, Arquivo) {
    let arquivo = Arquivo(Rc::new(RefCell::new(conteudo.map(str::to_owned))));
    let services = Services { store: Box::new(arquivo.clone()), compositor: Box::new(sparky), commands: Box::new(comandos) };
    let context = Context { home: PathBuf::from("/h"), about: About::default(), keymaps: Keymaps::parse(REGRAS), skin: Skin::default(), rows: 10 };
    (Settings::new(services, context), arquivo)
}

pub fn tecla(settings: &mut Settings, texto: &str) {
    let _ = update(settings, Message::Key(Key::Character(texto.into()), Modifiers::empty()));
}

pub fn enter(settings: &mut Settings) {
    let _ = update(settings, Message::Key(Key::Named(Named::Enter), Modifiers::empty()));
}

pub fn digitar(settings: &mut Settings, texto: &str) {
    let _ = update(settings, Message::Typed(texto.to_owned()));
    let _ = update(settings, Message::Submit);
}

pub fn salvo(arquivo: &Arquivo) -> String {
    arquivo.0.borrow().clone().unwrap_or_default()
}

