use std::fs;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use gio::prelude::*;

use crate::domain::bookmarks::{Bookmark, Target};
use crate::ports::BookmarkStore;

pub struct GtkBookmarks {
    file: PathBuf,
}

impl GtkBookmarks {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub fn from_env() -> Self {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_default();
        Self::new(config.join("gtk-3.0/bookmarks"))
    }
}

impl BookmarkStore for GtkBookmarks {
    fn load(&self) -> Result<Vec<Bookmark>, String> {
        match fs::read_to_string(&self.file) {
            Ok(contents) => Ok(contents.lines().filter_map(parse).collect()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn save(&self, bookmarks: &[Bookmark]) -> Result<(), String> {
        let parent = self.file.parent().ok_or("invalid bookmarks directory")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut staged =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        let text: String = bookmarks.iter().map(render).collect();
        staged
            .write_all(text.as_bytes())
            .map_err(|error| error.to_string())?;
        staged
            .persist(&self.file)
            .map(drop)
            .map_err(|error| error.to_string())
    }
}

fn parse(line: &str) -> Option<Bookmark> {
    let line = line.trim_end();
    if line.is_empty() {
        return None;
    }
    let (uri, label) = match line.split_once(' ') {
        Some((uri, label)) => (uri, Some(label.to_owned())),
        None => (line, None),
    };
    let target = match uri.starts_with("file://") {
        true => Target::Local(gio::File::for_uri(uri).path()?),
        false => Target::Remote(uri.to_owned()),
    };
    Some(Bookmark { target, label })
}

fn render(bookmark: &Bookmark) -> String {
    let uri = match &bookmark.target {
        Target::Local(path) => gio::File::for_path(path).uri().to_string(),
        Target::Remote(uri) => uri.clone(),
    };
    match &bookmark.label {
        Some(label) => format!("{uri} {label}\n"),
        None => format!("{uri}\n"),
    }
}
