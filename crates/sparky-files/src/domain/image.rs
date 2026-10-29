use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageMime {
    Png,
    Jpeg,
    Webp,
}

impl ImageMime {
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        match extension.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::Webp),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ImageMime;
    use std::path::Path;

    #[test]
    fn reconhece_imagens_por_extensao_sem_confundir_outros_arquivos() {
        assert_eq!(
            ImageMime::from_path(Path::new("foto.PNG")),
            Some(ImageMime::Png)
        );
        assert_eq!(
            ImageMime::from_path(Path::new("foto.jpeg")),
            Some(ImageMime::Jpeg)
        );
        assert_eq!(
            ImageMime::from_path(Path::new("foto.webp")),
            Some(ImageMime::Webp)
        );
        assert_eq!(ImageMime::from_path(Path::new("foto.txt")), None);
        assert_eq!(ImageMime::Png.as_str(), "image/png");
        assert_eq!(ImageMime::Jpeg.as_str(), "image/jpeg");
        assert_eq!(ImageMime::Webp.as_str(), "image/webp");
    }
}
