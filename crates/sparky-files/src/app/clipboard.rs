use std::path::PathBuf;

use crate::app::state::Files;
use crate::domain::image::ImageMime;
use crate::i18n;

impl Files {
    pub fn publish_image(&mut self, paths: &[PathBuf]) {
        let [path] = paths else {
            return;
        };
        if !path.is_file() {
            return;
        }
        let Some(mime) = ImageMime::from_path(path) else {
            return;
        };
        match self.services.image_clipboard.publish(path, mime) {
            Ok(()) => self.inform(i18n::text("image-copied")),
            Err(error) => self.fail(i18n::with("image-not-copied", "error", error.to_string())),
        }
    }
}
