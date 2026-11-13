use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::adapters::thumbnail_png::{self, Stamp};
use crate::domain::thumbnail::Thumbnail;
use crate::domain::thumbnail_key::{cache_name, file_uri};
use crate::ports::ThumbnailSource;

const WRITTEN: &str = "normal";
const READ: [&str; 4] = ["normal", "large", "x-large", "xx-large"];

pub struct SharedThumbnails<S> {
    root: PathBuf,
    source: S,
}

impl<S> SharedThumbnails<S> {
    pub fn new(root: PathBuf, source: S) -> Self {
        Self { root, source }
    }

    pub fn from_env(source: S) -> Self {
        let cache = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        Self::new(cache.join("thumbnails"), source)
    }

    fn cached(&self, name: &str, stamp: &Stamp) -> Option<Thumbnail> {
        READ.iter()
            .find_map(|size| thumbnail_png::read(&self.root.join(size).join(name), stamp))
    }
}

impl<S: ThumbnailSource> ThumbnailSource for SharedThumbnails<S> {
    fn load(&self, path: &Path) -> Option<Thumbnail> {
        let Some(stamp) = stamp(path) else {
            return self.source.load(path);
        };
        let name = cache_name(&stamp.uri);
        if let Some(thumbnail) = self.cached(&name, &stamp) {
            return Some(thumbnail);
        }
        let thumbnail = self.source.load(path)?;
        if !path.starts_with(&self.root) {
            let _ = thumbnail_png::write(&self.root.join(WRITTEN), &name, &stamp, &thumbnail);
        }
        Some(thumbnail)
    }
}

fn stamp(path: &Path) -> Option<Stamp> {
    let metadata = fs::metadata(path).ok()?;
    let mtime = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    let absolute = std::path::absolute(path).ok()?;
    Some(Stamp {
        uri: file_uri(&absolute),
        mtime: mtime.as_secs(),
        size: metadata.len(),
    })
}
