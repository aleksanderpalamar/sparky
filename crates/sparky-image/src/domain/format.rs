use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
}

impl Format {
    pub const ALL: [Format; 6] = [
        Format::Png,
        Format::Jpeg,
        Format::Webp,
        Format::Gif,
        Format::Bmp,
        Format::Tiff,
    ];

    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "png" => Some(Format::Png),
            "jpg" | "jpeg" => Some(Format::Jpeg),
            "webp" => Some(Format::Webp),
            "gif" => Some(Format::Gif),
            "bmp" => Some(Format::Bmp),
            "tif" | "tiff" => Some(Format::Tiff),
            _ => None,
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            Format::Png => "image/png",
            Format::Jpeg => "image/jpeg",
            Format::Webp => "image/webp",
            Format::Gif => "image/gif",
            Format::Bmp => "image/bmp",
            Format::Tiff => "image/tiff",
        }
    }
}
