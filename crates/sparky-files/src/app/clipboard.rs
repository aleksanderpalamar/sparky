use std::path::PathBuf;

use crate::app::state::Files;
use crate::domain::clip::{Clip, ClipOffer, Pasted};
use crate::domain::image::ImageMime;
use crate::i18n;

impl Files {
    pub fn publish_clip(&mut self) {
        let Some(clip) = self.clip.clone() else {
            return;
        };
        let image = match &clip {
            Clip::Copy(paths) => single_image(paths),
            Clip::Cut(_) => None,
        };
        let offer = ClipOffer {
            clip,
            image: image.clone(),
        };
        match self.services.clipboard.publish(&offer) {
            Ok(()) if image.is_some() => self.inform(i18n::text("image-copied")),
            Ok(()) => {}
            Err(error) => self.fail(i18n::with(
                "clipboard-not-published",
                "error",
                error.to_string(),
            )),
        }
    }

    pub fn clipboard_clip(&mut self) -> Option<Clip> {
        match self.services.clipboard.read() {
            Pasted::Files(clip) => {
                self.clip = Some(clip.clone());
                Some(clip)
            }
            Pasted::Other => {
                self.clip = None;
                None
            }
            Pasted::Unavailable => self.clip.clone(),
        }
    }
}

fn single_image(paths: &[PathBuf]) -> Option<(PathBuf, ImageMime)> {
    let [path] = paths else {
        return None;
    };
    let mime = ImageMime::from_path(path)?;
    path.is_file().then(|| (path.clone(), mime))
}
