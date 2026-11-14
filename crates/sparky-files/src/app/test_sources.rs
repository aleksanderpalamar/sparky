use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use sparky_storage::{Operation, Volume, VolumeSource};

use crate::domain::archive::{Extraction, extractable};
use crate::domain::clip::{ClipOffer, Pasted};
use crate::domain::icons::Icons;
use crate::domain::image::ImageMime;
use crate::domain::transfer::Progress;
use crate::ports::{Archiver, FileClipboard, IconSource, TrashDisks, TrashStore};

#[derive(Clone)]
pub struct AreaFalsa {
    pub ofertas: Rc<RefCell<Vec<ClipOffer>>>,
    pub limpezas: Rc<RefCell<usize>>,
    pub leitura: Rc<RefCell<Pasted>>,
    pub falha: bool,
}

impl Default for AreaFalsa {
    fn default() -> Self {
        Self {
            ofertas: Rc::default(),
            limpezas: Rc::default(),
            leitura: Rc::new(RefCell::new(Pasted::Unavailable)),
            falha: false,
        }
    }
}

impl AreaFalsa {
    pub fn imagens(&self) -> Vec<(PathBuf, ImageMime)> {
        self.ofertas
            .borrow()
            .iter()
            .filter_map(|offer| offer.image.clone())
            .collect()
    }
}

impl FileClipboard for AreaFalsa {
    fn publish(&self, offer: &ClipOffer) -> io::Result<()> {
        if self.falha {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "compositor sem data-control",
            ));
        }
        self.ofertas.borrow_mut().push(offer.clone());
        Ok(())
    }

    fn clear(&self) -> io::Result<()> {
        *self.limpezas.borrow_mut() += 1;
        Ok(())
    }

    fn read(&self) -> Pasted {
        self.leitura.borrow().clone()
    }
}

pub struct NoIcons;

impl IconSource for NoIcons {
    fn load(&mut self) -> Icons {
        Icons::default()
    }
}

pub struct NoVolumes;

impl VolumeSource for NoVolumes {
    fn run(&self, _operation: Operation) -> Result<Vec<Volume>, String> {
        Ok(Vec::new())
    }
}

pub struct NoDiskTrashes;

impl TrashDisks for NoDiskTrashes {
    fn others(&self) -> Vec<std::sync::Arc<dyn TrashStore>> {
        Vec::new()
    }
}

#[derive(Clone, Default)]
pub struct ArrastoFalso {
    pedidos: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    pub recusa: Option<String>,
}

impl ArrastoFalso {
    pub fn pedidos(&self) -> Vec<Vec<String>> {
        self.pedidos
            .lock()
            .map(|lista| lista.clone())
            .unwrap_or_default()
    }
}

impl crate::ports::Dragger for ArrastoFalso {
    fn start(&self, uris: Vec<String>) -> Result<(), String> {
        if let Ok(mut lista) = self.pedidos.lock() {
            lista.push(uris);
        }
        self.recusa.clone().map_or(Ok(()), Err)
    }
}

pub struct ArquivosComprimidos;

impl Archiver for ArquivosComprimidos {
    fn create(&self, _paths: &[PathBuf], target: &Path) -> io::Result<()> {
        fs::write(target, "arquivo de teste")
    }

    fn extract(
        &self,
        archive: &Path,
        into: &Path,
        advance: &mut dyn FnMut(Progress) -> bool,
    ) -> io::Result<Extraction> {
        let name = archive
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        let stem = name.as_deref().and_then(extractable).unwrap_or("saida");
        let target = into.join(stem);
        let progress = Progress {
            items_done: 1,
            items_total: 1,
            ..Progress::default()
        };
        if !advance(progress) {
            return Ok(Extraction::Cancelled);
        }
        fs::create_dir(&target)?;
        fs::write(target.join("conteudo.txt"), "extraído")?;
        Ok(Extraction::Done(target))
    }
}
