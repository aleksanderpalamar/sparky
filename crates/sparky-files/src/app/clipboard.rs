use std::path::PathBuf;

use crate::app::state::Files;
use crate::domain::image::ImageMime;

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
            Ok(()) => self.inform("Imagem copiada para a área de transferência".to_owned()),
            Err(error) => self.fail(format!(
                "Arquivo copiado no Sparky Files, mas a imagem não foi para a área de transferência: {error}"
            )),
        }
    }
}
