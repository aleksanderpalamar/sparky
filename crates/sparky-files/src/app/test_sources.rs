use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use sparky_storage::{Operation, Volume, VolumeSource};

use crate::domain::clip::{ClipOffer, Pasted};
use crate::domain::icons::Icons;
use crate::domain::image::ImageMime;
use crate::ports::{FileClipboard, IconSource, TrashDisks, TrashStore};

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
