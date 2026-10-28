use std::fs;
use std::path::Path;

use image::{ImageReader, Limits};

use crate::domain::thumbnail::{Thumbnail, supported};
use crate::ports::ThumbnailSource;

pub struct FileThumbnail;

impl ThumbnailSource for FileThumbnail {
    fn load(&self, path: &Path) -> Option<Thumbnail> {
        if !supported(path) || fs::metadata(path).ok()?.len() > 16 * 1024 * 1024 {
            return None;
        }
        let mut reader = ImageReader::open(path).ok()?.with_guessed_format().ok()?;
        let mut limits = Limits::default();
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        let image = reader.decode().ok()?.thumbnail(128, 128).to_rgba8();
        Some(Thumbnail {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{FileThumbnail, ThumbnailSource, fs};
    use image::{ImageBuffer, Rgb};

    #[test]
    fn decodifica_miniatura_limitada_e_ignora_arquivo_corrompido() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("imagem.png");
        ImageBuffer::from_pixel(512, 256, Rgb([10u8, 20, 30]))
            .save(&path)
            .unwrap();
        let preview = FileThumbnail.load(&path).unwrap();
        assert_eq!((preview.width, preview.height), (128, 64));
        assert_eq!(preview.rgba.len(), 128 * 64 * 4);
        fs::write(&path, b"quebrado").unwrap();
        assert!(FileThumbnail.load(&path).is_none());
    }
}
