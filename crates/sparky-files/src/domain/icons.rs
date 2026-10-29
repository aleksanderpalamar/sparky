use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::entry::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IconKind {
    Folder,
    Home,
    Desktop,
    Documents,
    Downloads,
    Pictures,
    Music,
    Videos,
    File,
    Text,
    Code,
    Pdf,
    Image,
    Audio,
    Video,
    Archive,
    Document,
    Spreadsheet,
    Presentation,
    Drive,
    Removable,
    Phone,
    Eject,
    Trash,
}

pub const ALL: [IconKind; 24] = [
    IconKind::Folder,
    IconKind::Home,
    IconKind::Desktop,
    IconKind::Documents,
    IconKind::Downloads,
    IconKind::Pictures,
    IconKind::Music,
    IconKind::Videos,
    IconKind::File,
    IconKind::Text,
    IconKind::Code,
    IconKind::Pdf,
    IconKind::Image,
    IconKind::Audio,
    IconKind::Video,
    IconKind::Archive,
    IconKind::Document,
    IconKind::Spreadsheet,
    IconKind::Presentation,
    IconKind::Drive,
    IconKind::Removable,
    IconKind::Phone,
    IconKind::Eject,
    IconKind::Trash,
];

pub type Icons = BTreeMap<IconKind, PathBuf>;

impl IconKind {
    pub fn for_entry(entry: &Entry, place: Option<&str>) -> Self {
        if entry.is_directory() {
            return Self::place(place.unwrap_or_default());
        }
        let extension = Path::new(&entry.name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_lowercase();
        match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "avif" | "tif" | "tiff" => {
                Self::Image
            }
            "mp3" | "flac" | "ogg" | "wav" | "m4a" | "opus" | "aac" => Self::Audio,
            "mp4" | "mkv" | "webm" | "avi" | "mov" | "m4v" | "mpeg" => Self::Video,
            "pdf" => Self::Pdf,
            "zip" | "tar" | "gz" | "xz" | "bz2" | "zst" | "7z" | "rar" | "deb" | "rpm" => {
                Self::Archive
            }
            "doc" | "docx" | "odt" | "rtf" => Self::Document,
            "xls" | "xlsx" | "ods" | "csv" => Self::Spreadsheet,
            "ppt" | "pptx" | "odp" => Self::Presentation,
            "rs" | "c" | "h" | "cpp" | "py" | "js" | "ts" | "tsx" | "html" | "css" | "sh"
            | "json" | "toml" | "yaml" | "yml" => Self::Code,
            "txt" | "md" | "log" | "conf" | "ini" => Self::Text,
            _ => Self::File,
        }
    }

    pub fn place(label: &str) -> Self {
        match label {
            "Pasta pessoal" => Self::Home,
            "Área de trabalho" => Self::Desktop,
            "Documentos" => Self::Documents,
            "Downloads" => Self::Downloads,
            "Imagens" => Self::Pictures,
            "Músicas" => Self::Music,
            "Vídeos" => Self::Videos,
            _ => Self::Folder,
        }
    }

    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::Folder => &["folder"],
            Self::Home => &["user-home", "folder"],
            Self::Desktop => &["user-desktop", "folder"],
            Self::Documents => &["folder-documents", "folder"],
            Self::Downloads => &["folder-download", "folder"],
            Self::Pictures => &["folder-pictures", "folder"],
            Self::Music => &["folder-music", "folder"],
            Self::Videos => &["folder-videos", "folder"],
            Self::File | Self::Text => &["text-x-generic"],
            Self::Code => &["text-x-script", "text-x-generic"],
            Self::Pdf => &["application-pdf", "x-office-document", "text-x-generic"],
            Self::Image => &["image-x-generic", "text-x-generic"],
            Self::Audio => &["audio-x-generic", "text-x-generic"],
            Self::Video => &["video-x-generic", "text-x-generic"],
            Self::Archive => &[
                "package-x-generic",
                "application-x-archive",
                "text-x-generic",
            ],
            Self::Document => &["x-office-document", "text-x-generic"],
            Self::Spreadsheet => &["x-office-spreadsheet", "text-x-generic"],
            Self::Presentation => &["x-office-presentation", "text-x-generic"],
            Self::Drive => &["drive-harddisk", "drive-removable-media"],
            Self::Removable => &["drive-removable-media", "drive-harddisk"],
            Self::Phone => &["phone", "multimedia-player", "drive-removable-media"],
            Self::Eject => &["media-eject", "media-eject-symbolic"],
            Self::Trash => &["user-trash", "user-trash-full"],
        }
    }
}
