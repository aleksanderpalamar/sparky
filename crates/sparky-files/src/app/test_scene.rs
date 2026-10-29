use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use sparky_theme::Skin;
use tempfile::TempDir;

use super::test_sources::{CopiedImages, NoIcons, NoVolumes};
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::app::message::Message;
use crate::app::state::{Files, Services, Setup};
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

pub struct Cena {
    pub dir: TempDir,
    pub files: Files,
    pub imagens: CopiedImages,
    abertos: Abertos,
}

impl Cena {
    pub fn nova() -> Self {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("sem pasta temporária")
        };
        let work = dir.path().join("work");
        let preparado = fs::create_dir_all(work.join("Fotos"))
            .and_then(|()| fs::write(work.join("a.txt"), "oi"));
        assert!(preparado.is_ok());
        let abertos = Abertos::default();
        let imagens = CopiedImages::default();
        let services = Services {
            disk: Box::new(LocalDisk::new(HomeTrash::new(dir.path().join("Trash")))),
            opener: Box::new(abertos.clone()),
            icons: Box::new(NoIcons),
            volumes: Arc::new(NoVolumes),
            image_clipboard: Box::new(imagens.clone()),
        };
        let setup = Setup {
            start: work.clone(),
            home: dir.path().to_path_buf(),
            places: Vec::new(),
            skin: Skin::default(),
            rows: 10,
        };
        Self {
            files: Files::new(setup, services),
            dir,
            imagens,
            abertos,
        }
    }

    pub fn tecla(&mut self, texto: &str) {
        let _ = update(
            &mut self.files,
            Message::Key(Key::Character(texto.into()), Modifiers::empty()),
        );
    }

    pub fn nomeada(&mut self, named: Named) {
        let _ = update(
            &mut self.files,
            Message::Key(Key::Named(named), Modifiers::empty()),
        );
    }

    pub fn mandar(&mut self, message: Message) {
        let _ = update(&mut self.files, message);
    }

    pub fn work(&self) -> PathBuf {
        self.dir.path().join("work")
    }

    pub fn selecionado(&self) -> Option<String> {
        self.files
            .listing
            .selected()
            .map(|entry| entry.name.clone())
    }

    pub fn abertos(&self) -> Vec<PathBuf> {
        self.abertos.0.borrow().clone()
    }
}
