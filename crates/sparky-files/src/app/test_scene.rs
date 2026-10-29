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
use crate::adapters::thumbnail::FileThumbnail;
use crate::app::message::Message;
use crate::app::state::{Files, Services, Setup};
use crate::app::update::update;
use crate::domain::application::Application;
use crate::ports::{Archiver, Opener};

#[derive(Clone, Default)]
struct Abertos(Rc<RefCell<Vec<PathBuf>>>);

impl Opener for Abertos {
    fn open(&self, path: &Path) -> io::Result<()> {
        self.0.borrow_mut().push(path.to_path_buf());
        Ok(())
    }

    fn applications(&self, _path: &Path) -> Vec<Application> {
        vec![Application {
            id: "teste.desktop".to_owned(),
            name: "Teste".to_owned(),
        }]
    }

    fn open_with(&self, path: &Path, _id: &str) -> io::Result<()> {
        self.open(path)
    }

    fn email(&self, paths: &[PathBuf]) -> io::Result<()> {
        self.0.borrow_mut().extend(paths.iter().cloned());
        Ok(())
    }
}

struct ArquivosComprimidos;

impl Archiver for ArquivosComprimidos {
    fn create(&self, _paths: &[PathBuf], target: &Path) -> io::Result<()> {
        fs::write(target, "arquivo de teste")
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
        let trash = HomeTrash::new(dir.path().join("Trash"));
        let services = Services {
            disk: Box::new(LocalDisk::new(trash.clone())),
            opener: Box::new(abertos.clone()),
            icons: Box::new(NoIcons),
            volumes: Arc::new(NoVolumes),
            archiver: Arc::new(ArquivosComprimidos),
            thumbnails: Arc::new(FileThumbnail),
            trash: Arc::new(trash),
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
