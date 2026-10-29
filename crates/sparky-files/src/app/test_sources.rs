use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use sparky_storage::{Operation, Volume, VolumeSource};

use crate::domain::icons::Icons;
use crate::domain::image::ImageMime;
use crate::ports::{IconSource, ImageClipboard};

#[derive(Clone, Default)]
pub struct CopiedImages(pub Rc<RefCell<Vec<(PathBuf, ImageMime)>>>);

impl ImageClipboard for CopiedImages {
    fn publish(&self, path: &Path, mime: ImageMime) -> io::Result<()> {
        self.0.borrow_mut().push((path.to_path_buf(), mime));
        Ok(())
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
