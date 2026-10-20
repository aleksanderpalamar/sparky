use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use iced::event::{Event, Status};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Location, Modifiers};
use sparky_theme::Skin;
use tempfile::TempDir;

use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::app::events::keyboard_message;
use crate::app::message::Message;
use crate::app::state::{Files, Mode, Notice, Services, Setup};
use crate::app::update::update;
use crate::ports::Opener;

#[derive(Clone, Default)]
struct Abertos(Rc<RefCell<Vec<PathBuf>>>);

impl Opener for Abertos {
    fn open(&self, path: &Path) -> io::Result<()> {
        self.0.borrow_mut().push(path.to_path_buf());
        Ok(())
    }
}

struct Cena {
    dir: TempDir,
    files: Files,
    abertos: Abertos,
}

impl Cena {
    fn nova() -> Self {
        let Ok(dir) = tempfile::tempdir() else { panic!("sem pasta temporária") };
        let work = dir.path().join("work");
        let preparado = fs::create_dir_all(work.join("Fotos")).and_then(|()| fs::write(work.join("a.txt"), "oi"));
        assert!(preparado.is_ok());
        let abertos = Abertos::default();
        let services = Services {
            disk: Box::new(LocalDisk::new(HomeTrash::new(dir.path().join("Trash")))),
            opener: Box::new(abertos.clone()),
        };
        let setup = Setup { start: work.clone(), home: dir.path().to_path_buf(), places: Vec::new(), skin: Skin::default(), rows: 10 };
        Self { files: Files::new(setup, services), dir, abertos }
    }

    fn tecla(&mut self, texto: &str) {
        let _ = update(&mut self.files, Message::Key(Key::Character(texto.into()), Modifiers::empty()));
    }

    fn nomeada(&mut self, named: Named) {
        let _ = update(&mut self.files, Message::Key(Key::Named(named), Modifiers::empty()));
    }

    fn mandar(&mut self, message: Message) {
        let _ = update(&mut self.files, message);
    }

    fn work(&self) -> PathBuf {
        self.dir.path().join("work")
    }

    fn selecionado(&self) -> Option<String> {
        self.files.listing.selected().map(|entry| entry.name.clone())
    }
}

#[test]
fn entrar_e_voltar_deixa_a_pasta_de_onde_veio_selecionada() {
    let mut cena = Cena::nova();
    cena.tecla("l");
    assert_eq!(cena.files.location, cena.work().join("Fotos"));
    cena.tecla("h");
    assert_eq!(cena.files.location, cena.work());
    assert_eq!(cena.selecionado().as_deref(), Some("Fotos"));
}

#[test]
fn criar_pasta_pelo_prompt_seleciona_a_nova() {
    let mut cena = Cena::nova();
    cena.tecla("a");
    assert_eq!(cena.files.mode, Mode::Create);
    cena.mandar(Message::Typed("Projetos/".to_owned()));
    cena.mandar(Message::Submit);
    assert!(cena.work().join("Projetos").is_dir());
    assert_eq!(cena.selecionado().as_deref(), Some("Projetos"));
    assert_eq!(cena.files.mode, Mode::Browse);
}

#[test]
fn renomear_comeca_com_o_nome_atual() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("r");
    assert_eq!(cena.files.input, "a.txt");
    cena.mandar(Message::Typed("b.txt".to_owned()));
    cena.mandar(Message::Submit);
    assert!(cena.work().join("b.txt").exists());
    assert_eq!(cena.files.notice, Some(Notice::Info("Renomeado para b.txt".to_owned())));
}

#[test]
fn copiar_e_colar_na_mesma_pasta_duplica() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("y");
    cena.tecla("p");
    assert!(cena.work().join("a (cópia).txt").exists());
    assert_eq!(cena.selecionado().as_deref(), Some("a (cópia).txt"));
}

#[test]
fn a_lixeira_so_age_depois_da_confirmacao() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.tecla("d");
    assert_eq!(cena.files.mode, Mode::ConfirmTrash(vec!["a.txt".to_owned()]));
    cena.tecla("n");
    assert!(cena.work().join("a.txt").exists());
    cena.tecla("d");
    cena.nomeada(Named::Enter);
    assert!(!cena.work().join("a.txt").exists());
    assert!(cena.dir.path().join("Trash/files/a.txt").exists());
}

#[test]
fn o_filtro_reduz_a_lista_e_esc_limpa() {
    let mut cena = Cena::nova();
    cena.tecla("/");
    cena.mandar(Message::Typed("fo".to_owned()));
    assert_eq!(cena.files.listing.len(), 1);
    cena.mandar(Message::Escape);
    assert_eq!(cena.files.listing.len(), 2);
    assert_eq!(cena.files.mode, Mode::Browse);
}

#[test]
fn abrir_um_arquivo_chama_o_aplicativo_padrao() {
    let mut cena = Cena::nova();
    cena.tecla("G");
    cena.nomeada(Named::Enter);
    assert_eq!(*cena.abertos.0.borrow(), vec![cena.work().join("a.txt")]);
}

#[test]
fn marcar_com_espaco_e_copiar_leva_todos_os_marcados() {
    let mut cena = Cena::nova();
    cena.nomeada(Named::Space);
    cena.nomeada(Named::Space);
    cena.tecla("y");
    assert_eq!(cena.files.clip.as_ref().map(|clip| clip.paths().len()), Some(2));
    assert_eq!(cena.files.listing.marked_count(), 0);
}

fn tecla_pressionada(key: Key) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat: false,
    })
}

#[test]
fn esc_chega_mesmo_com_a_caixa_de_texto_em_foco_e_o_resto_nao() {
    let window = iced::window::Id::unique();
    let esc = keyboard_message(tecla_pressionada(Key::Named(Named::Escape)), Status::Captured, window);
    let letra = keyboard_message(tecla_pressionada(Key::Character("j".into())), Status::Captured, window);
    let livre = keyboard_message(tecla_pressionada(Key::Character("j".into())), Status::Ignored, window);
    assert!(matches!(esc, Some(Message::Escape)));
    assert!(letra.is_none());
    assert!(matches!(livre, Some(Message::Key(..))));
}
