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

use super::test_sources::{
    AreaFalsa, ArquivosComprimidos, ArrastoFalso, NoDiskTrashes, NoIcons, NoVolumes,
};
use crate::adapters::bookmark_file::BookmarkFile;
use crate::adapters::finder::TreeFinder;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::adapters::sidebar_source::DesktopSidebar;
use crate::adapters::thumbnail::FileThumbnail;
use crate::adapters::transfer::LocalTransfer;
use crate::app::message::Message;
use crate::app::state::{Files, Services, Setup};
use crate::app::update::update;
use crate::domain::application::{AppChoices, Application, Remember};
use crate::ports::Opener;

type Escolhas = Rc<RefCell<Vec<(String, Remember)>>>;

#[derive(Clone, Default)]
struct Abertos(Rc<RefCell<Vec<PathBuf>>>, Escolhas);

fn aplicativo(id: &str, name: &str) -> Application {
    Application {
        id: id.to_owned(),
        name: name.to_owned(),
        icon: None,
    }
}

impl Opener for Abertos {
    fn open(&self, path: &Path) -> io::Result<()> {
        self.0.borrow_mut().push(path.to_path_buf());
        Ok(())
    }

    fn applications(&self, _path: &Path) -> AppChoices {
        AppChoices {
            kind: "Texto".to_owned(),
            recommended: vec![aplicativo("teste.desktop", "Teste")],
            others: vec![aplicativo("outro.desktop", "Outro Editor")],
        }
    }

    fn open_with(&self, path: &Path, id: &str, remember: Remember) -> io::Result<()> {
        self.1.borrow_mut().push((id.to_owned(), remember));
        self.open(path)
    }

    fn email(&self, paths: &[PathBuf]) -> io::Result<()> {
        self.0.borrow_mut().extend(paths.iter().cloned());
        Ok(())
    }
}

pub struct Cena {
    pub dir: TempDir,
    pub files: Files,
    pub area: AreaFalsa,
    pub arrastos: ArrastoFalso,
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
        let area = AreaFalsa::default();
        let arrastos = ArrastoFalso::default();
        let trash = HomeTrash::new(dir.path().join("Trash"));
        let services = Services {
            disk: Box::new(LocalDisk::new(trash.clone())),
            opener: Box::new(abertos.clone()),
            icons: Box::new(NoIcons),
            volumes: Arc::new(NoVolumes),
            archiver: Arc::new(ArquivosComprimidos),
            thumbnails: Arc::new(FileThumbnail),
            trash: Arc::new(trash.clone()),
            trash_disks: Arc::new(NoDiskTrashes),
            finder: Arc::new(TreeFinder::new(100)),
            dragger: Arc::new(arrastos.clone()),
            bookmarks: Arc::new(BookmarkFile::new(
                dir.path().join("config/sparky/bookmarks"),
            )),
            clipboard: Box::new(area.clone()),
            sidebar: Arc::new(DesktopSidebar::new(dir.path().join("data"))),
            transfer: Arc::new(LocalTransfer::new(trash.clone(), None)),
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
            area,
            arrastos,
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

    pub fn concluir(&mut self, message: Message) {
        let task = update(&mut self.files, message);
        super::test_tasks::concluir(&mut self.files, task, 1000);
    }

    pub fn tecla_concluida(&mut self, texto: &str) {
        self.concluir(Message::Key(
            Key::Character(texto.into()),
            Modifiers::empty(),
        ));
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

    pub fn escolhas(&self) -> Vec<(String, Remember)> {
        self.abertos.1.borrow().clone()
    }

    pub fn abertos(&self) -> Vec<PathBuf> {
        self.abertos.0.borrow().clone()
    }
}
