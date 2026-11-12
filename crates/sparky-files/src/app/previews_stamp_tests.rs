use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use iced::Size;

use super::message::Message;
use super::test_scene::Cena;
use super::test_tasks::concluir;
use super::update::update;
use crate::adapters::home_trash::HomeTrash;
use crate::adapters::local_disk::LocalDisk;
use crate::domain::entry::Entry;
use crate::domain::properties::Properties;
use crate::domain::thumbnail::Thumbnail;
use crate::ports::{Disk, ThumbnailSource};

struct ConsultasContadas {
    disco: LocalDisk,
    consultas: Arc<AtomicUsize>,
}

impl Disk for ConsultasContadas {
    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>> {
        self.disco.list(dir)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.disco.canonicalize(path)
    }

    fn exists(&self, path: &Path) -> bool {
        self.disco.exists(path)
    }

    fn stamp(&self, dir: &Path) -> Option<SystemTime> {
        if dir.extension().is_some() {
            self.consultas.fetch_add(1, Ordering::SeqCst);
        }
        self.disco.stamp(dir)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        self.disco.create_dir(path)
    }

    fn create_file(&self, path: &Path) -> io::Result<()> {
        self.disco.create_file(path)
    }

    fn relocate(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.disco.relocate(from, to)
    }

    fn trash(&self, path: &Path) -> io::Result<()> {
        self.disco.trash(path)
    }

    fn properties(&self, path: &Path) -> io::Result<Properties> {
        self.disco.properties(path)
    }
}

struct Fixa;

impl ThumbnailSource for Fixa {
    fn load(&self, _path: &Path) -> Option<Thumbnail> {
        Some(Thumbnail {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        })
    }
}

#[test]
fn previas_usam_a_data_da_listagem_sem_consultar_cada_imagem() {
    let mut scene = Cena::nova();
    let consultas = Arc::new(AtomicUsize::new(0));
    let lixeira = HomeTrash::new(scene.dir.path().join("Trash"));
    scene.files.services.disk = Box::new(ConsultasContadas {
        disco: LocalDisk::new(lixeira),
        consultas: consultas.clone(),
    });
    scene.files.services.thumbnails = Arc::new(Fixa);
    scene.files.resize(Size::new(900.0, 560.0));
    let imagens: Vec<_> = (0..6)
        .map(|index| scene.work().join(format!("foto-{index}.png")))
        .collect();
    for imagem in &imagens {
        fs::write(imagem, "png").unwrap();
    }
    scene.files.refresh();
    let task = scene.files.request_previews();
    concluir(&mut scene.files, task, 100);
    for _ in 0..20 {
        let _ = update(&mut scene.files, Message::Scrolled(0));
    }
    assert!(
        imagens
            .iter()
            .all(|imagem| scene.files.previews.handle(imagem).is_some())
    );
    assert_eq!(consultas.load(Ordering::SeqCst), 0);
}
